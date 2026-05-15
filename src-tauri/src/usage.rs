use crate::model::{ModelDailyTokenUsage, RefreshResult};
use chrono::{DateTime, Local};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub source: String,
    pub usage_id: String,
    pub created_at: Option<String>,
    pub fallback_at: Option<String>,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub message_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UsageBucketKey {
    source: String,
    date: String,
    model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UsageUnitKey {
    source: String,
    date: String,
    usage_id: String,
}

pub fn date_from_rfc3339(value: &str) -> Option<String> {
    let value = value.trim();
    if is_date_only(value) {
        return Some(value.to_string());
    }

    DateTime::parse_from_rfc3339(value).ok().map(|timestamp| {
        timestamp
            .with_timezone(&Local)
            .format("%Y-%m-%d")
            .to_string()
    })
}

fn is_date_only(value: &str) -> bool {
    value.len() == 10
        && value
            .as_bytes()
            .iter()
            .enumerate()
            .all(|(index, byte)| match index {
                4 | 7 => *byte == b'-',
                _ => byte.is_ascii_digit(),
            })
}

fn usage_unit_id(usage_id: &str) -> String {
    usage_id
        .split_once("#model:")
        .map(|(unit_id, _)| unit_id.to_string())
        .unwrap_or_else(|| usage_id.to_string())
}

pub fn refresh_token_usage_for_source(source: &str) -> Result<RefreshResult, String> {
    let previous = crate::index::read_token_usage()?
        .by_provider
        .into_iter()
        .find(|item| item.source == source)
        .map(|item| item.session_count)
        .unwrap_or(0);

    let session_records = crate::providers::ProviderRegistry::bootstrap()
        .resolve_or_default(Some(source))
        .list_sessions(None)?;
    let session_refresh = crate::index::refresh_source(source, session_records)?;

    let records = load_provider_usage_records(source)?;
    let aggregated = aggregate_created_day(records);

    crate::index::replace_usage_daily_model(source, aggregated)?;

    Ok(RefreshResult {
        source: source.to_string(),
        refreshed_at_ms: crate::index::now_ms(),
        previous_count: previous,
        current_count: session_refresh.current_count,
    })
}

pub fn load_provider_usage_records(source: &str) -> Result<Vec<UsageRecord>, String> {
    match source {
        "claude" => crate::providers::claude::list_usage_records(None),
        "codex" => crate::providers::codex::list_usage_records_from_db(None),
        "deepseek" => crate::providers::deepseek::list_usage_records(None),
        other => Err(format!("Unsupported usage source: {other}")),
    }
}

pub fn aggregate_created_day(records: Vec<UsageRecord>) -> Vec<ModelDailyTokenUsage> {
    let mut buckets: BTreeMap<UsageBucketKey, (u64, u64, u64, u64, u64)> = BTreeMap::new();
    let mut counted_units: BTreeSet<UsageUnitKey> = BTreeSet::new();

    for record in records {
        if record.total_tokens == 0 {
            continue;
        }

        let Some(date) = record
            .created_at
            .as_deref()
            .and_then(date_from_rfc3339)
            .or_else(|| record.fallback_at.as_deref().and_then(date_from_rfc3339))
        else {
            continue;
        };

        let unit_key = UsageUnitKey {
            source: record.source.clone(),
            date: date.clone(),
            usage_id: usage_unit_id(&record.usage_id),
        };
        let count_unit = counted_units.insert(unit_key);
        let key = UsageBucketKey {
            source: record.source,
            date,
            model: if record.model.trim().is_empty() {
                "unknown".to_string()
            } else {
                record.model
            },
        };

        let entry = buckets.entry(key).or_insert((0, 0, 0, 0, 0));
        entry.0 += record.input_tokens;
        entry.1 += record.output_tokens;
        entry.2 += record.total_tokens;
        if count_unit {
            entry.3 += 1;
            entry.4 += record.message_count;
        }
    }

    buckets
        .into_iter()
        .map(
            |(key, (input_tokens, output_tokens, total_tokens, session_count, message_count))| ModelDailyTokenUsage {
                date: key.date,
                source: key.source,
                model: key.model,
                input_tokens,
                output_tokens,
                total_tokens,
                session_count,
                message_count,
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_created_day_groups_by_created_date_source_and_model() {
        let rows = aggregate_created_day(vec![
            UsageRecord {
                source: "codex".to_string(),
                usage_id: "thread-a".to_string(),
                created_at: Some("2026-05-01T10:00:00Z".to_string()),
                fallback_at: Some("2026-05-03T10:00:00Z".to_string()),
                model: "gpt-5.5".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 100,
                message_count: 2,
            },
            UsageRecord {
                source: "codex".to_string(),
                usage_id: "thread-b".to_string(),
                created_at: Some("2026-05-01T12:00:00Z".to_string()),
                fallback_at: None,
                model: "gpt-5.5".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 50,
                message_count: 1,
            },
            UsageRecord {
                source: "deepseek".to_string(),
                usage_id: "session-c".to_string(),
                created_at: Some("2026-05-02T00:00:00Z".to_string()),
                fallback_at: None,
                model: "deepseek-v4-pro".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 25,
                message_count: 4,
            },
        ]);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].date, "2026-05-01");
        assert_eq!(rows[0].source, "codex");
        assert_eq!(rows[0].model, "gpt-5.5");
        assert_eq!(rows[0].total_tokens, 150);
        assert_eq!(rows[0].session_count, 2);
        assert_eq!(rows[0].message_count, 3);
        assert_eq!(rows[1].date, "2026-05-02");
        assert_eq!(rows[1].source, "deepseek");
        assert_eq!(rows[1].model, "deepseek-v4-pro");
        assert_eq!(rows[1].total_tokens, 25);
        assert_eq!(rows[1].session_count, 1);
        assert_eq!(rows[1].message_count, 4);
    }

    #[test]
    fn aggregate_created_day_counts_each_usage_id_once_per_source_date() {
        let rows = aggregate_created_day(vec![
            UsageRecord {
                source: "claude".to_string(),
                usage_id: "session-a#model:opus".to_string(),
                created_at: Some("2026-05-01T10:00:00Z".to_string()),
                fallback_at: None,
                model: "opus".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 100,
                message_count: 3,
            },
            UsageRecord {
                source: "claude".to_string(),
                usage_id: "session-a#model:sonnet".to_string(),
                created_at: Some("2026-05-01T10:00:00Z".to_string()),
                fallback_at: None,
                model: "sonnet".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 50,
                message_count: 3,
            },
        ]);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].model, "opus");
        assert_eq!(rows[0].session_count, 1);
        assert_eq!(rows[0].message_count, 3);
        assert_eq!(rows[1].model, "sonnet");
        assert_eq!(rows[1].session_count, 0);
        assert_eq!(rows[1].message_count, 0);
    }

    #[test]
    fn aggregate_created_day_uses_fallback_and_skips_unusable_records() {
        let rows = aggregate_created_day(vec![
            UsageRecord {
                source: "claude".to_string(),
                usage_id: "fallback".to_string(),
                created_at: None,
                fallback_at: Some("2026-05-04T00:00:00Z".to_string()),
                model: "  ".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 12,
                message_count: 3,
            },
            UsageRecord {
                source: "claude".to_string(),
                usage_id: "zero".to_string(),
                created_at: Some("2026-05-04T00:00:00Z".to_string()),
                fallback_at: None,
                model: "sonnet".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
                message_count: 9,
            },
            UsageRecord {
                source: "claude".to_string(),
                usage_id: "undated".to_string(),
                created_at: Some("not-a-date".to_string()),
                fallback_at: None,
                model: "sonnet".to_string(),
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 10,
                message_count: 1,
            },
        ]);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].date, "2026-05-04");
        assert_eq!(rows[0].source, "claude");
        assert_eq!(rows[0].model, "unknown");
        assert_eq!(rows[0].total_tokens, 12);
        assert_eq!(rows[0].session_count, 1);
        assert_eq!(rows[0].message_count, 3);
    }

    #[test]
    fn date_from_rfc3339_requires_date_prefix_digits_and_boundary() {
        assert_eq!(
            date_from_rfc3339("2026-05-12"),
            Some("2026-05-12".to_string())
        );
        assert_eq!(date_from_rfc3339("2026-ab-12T03:00:00Z"), None);
        assert_eq!(date_from_rfc3339("2026-05-12foo"), None);
        assert_eq!(date_from_rfc3339("20260512"), None);
    }

    #[test]
    fn date_from_rfc3339_converts_timestamp_to_local_calendar_day() {
        let timestamp = "2026-05-12T18:30:00Z";
        let expected = chrono::DateTime::parse_from_rfc3339(timestamp)
            .expect("parse timestamp")
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string();

        assert_eq!(date_from_rfc3339(timestamp), Some(expected));
    }
}
