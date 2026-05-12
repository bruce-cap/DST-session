//! Maintains the app-owned SQLite session index.

use crate::model::{
    DailyTokenUsage, ModelDailyTokenUsage, ModelTokenUsage, ProviderTokenUsage, RefreshResult,
    SessionRecord, SourceState, TokenUsageSummary,
};
use crate::paths::app_index_path;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn read_sessions(source: &str) -> Result<Vec<SessionRecord>, String> {
    let conn = open_index()?;
    let mut stmt = conn
        .prepare(
            "SELECT source, id, short_id, title, preview, created_at, updated_at,
                    message_count, total_tokens, model, workspace, mode, path, invalid_reason
             FROM sessions
             WHERE source = ?1 AND deleted_at_ms IS NULL
             ORDER BY COALESCE(updated_at, '') DESC",
        )
        .map_err(|error| format!("准备读取 session 索引失败: {error}"))?;

    let rows = stmt
        .query_map([source], session_record_from_row)
        .map_err(|error| format!("读取 session 索引失败: {error}"))?;

    let mut records = Vec::new();
    for row in rows {
        records.push(row.map_err(|error| format!("读取 session 索引行失败: {error}"))?);
    }
    Ok(records)
}

pub fn read_session(source: &str, session_id: &str) -> Result<SessionRecord, String> {
    let conn = open_index()?;
    conn.query_row(
        "SELECT source, id, short_id, title, preview, created_at, updated_at,
                message_count, total_tokens, model, workspace, mode, path, invalid_reason
         FROM sessions
         WHERE source = ?1 AND id = ?2 AND deleted_at_ms IS NULL",
        params![source, session_id],
        session_record_from_row,
    )
    .optional()
    .map_err(|error| format!("读取 session 索引失败 {source}:{session_id}: {error}"))?
    .ok_or_else(|| format!("未找到 session: {session_id}"))
}

pub fn refresh_source(source: &str, records: Vec<SessionRecord>) -> Result<RefreshResult, String> {
    let mut conn = open_index()?;
    let refreshed_at_ms = now_ms();
    let tx = conn
        .transaction()
        .map_err(|error| format!("打开 session 索引事务失败: {error}"))?;

    let previous_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE source = ?1 AND deleted_at_ms IS NULL",
            [source],
            |row| row.get(0),
        )
        .map_err(|error| format!("读取刷新前 session 数量失败: {error}"))?;

    tx.execute(
        "UPDATE sessions SET deleted_at_ms = ?2 WHERE source = ?1 AND deleted_at_ms IS NULL",
        params![source, refreshed_at_ms],
    )
    .map_err(|error| format!("标记旧 session 索引失败: {error}"))?;

    for record in &records {
        upsert_session(&tx, record, refreshed_at_ms)?;
    }

    let previous_state = read_source_state_from_conn(&tx, source)?;
    let next_state = SourceState {
        source: source.to_string(),
        last_refresh_at_ms: Some(refreshed_at_ms),
        last_success_at_ms: Some(refreshed_at_ms),
        last_error: None,
        refresh_watermark: previous_state.and_then(|state| state.refresh_watermark),
    };
    upsert_source_state(&tx, &next_state)?;

    tx.commit()
        .map_err(|error| format!("提交 session 索引刷新失败: {error}"))?;

    Ok(RefreshResult {
        source: source.to_string(),
        refreshed_at_ms,
        previous_count: previous_count.max(0) as u64,
        current_count: records.len() as u64,
    })
}

pub fn read_source_state(source: &str) -> Result<Option<SourceState>, String> {
    let conn = open_index()?;
    read_source_state_from_conn(&conn, source)
}

pub fn read_token_usage() -> Result<TokenUsageSummary, String> {
    let conn = open_index()?;
    let by_provider = read_usage_by_provider(&conn)?;
    let by_day = read_usage_by_day(&conn)?;
    let by_model = read_usage_by_model(&conn)?;
    let by_model_by_day = read_usage_by_model_by_day(&conn)?;
    let total_tokens = by_provider.iter().map(|item| item.total_tokens).sum();
    let total_sessions = by_provider.iter().map(|item| item.session_count).sum();
    let total_messages = by_provider.iter().map(|item| item.message_count).sum();

    Ok(TokenUsageSummary {
        total_tokens,
        total_sessions,
        total_messages,
        by_provider,
        by_day,
        by_model,
        by_model_by_day,
    })
}

pub fn replace_usage_daily_model(
    source: &str,
    rows: Vec<ModelDailyTokenUsage>,
) -> Result<(), String> {
    let mut conn = open_index()?;
    replace_usage_daily_model_in_conn(&mut conn, source, rows)
}

pub fn record_refresh_error(source: &str, error: &str) -> Result<(), String> {
    let conn = open_index()?;
    let previous = read_source_state_from_conn(&conn, source)?;
    let state = SourceState {
        source: source.to_string(),
        last_refresh_at_ms: Some(now_ms()),
        last_success_at_ms: previous.as_ref().and_then(|state| state.last_success_at_ms),
        last_error: Some(error.to_string()),
        refresh_watermark: previous.and_then(|state| state.refresh_watermark),
    };
    upsert_source_state(&conn, &state)
}

fn session_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionRecord> {
    Ok(SessionRecord {
        source: row.get(0)?,
        id: row.get(1)?,
        short_id: row.get(2)?,
        title: row.get(3)?,
        preview: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        message_count: row.get::<_, i64>(7)?.max(0) as u64,
        total_tokens: row.get::<_, i64>(8)?.max(0) as u64,
        model: row.get(9)?,
        workspace: row.get(10)?,
        mode: row.get(11)?,
        path: row.get(12)?,
        invalid_reason: row.get(13)?,
    })
}

fn read_usage_by_provider(conn: &Connection) -> Result<Vec<ProviderTokenUsage>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                source,
                COALESCE(SUM(total_tokens), 0) AS total_tokens,
                COALESCE(SUM(session_count), 0) AS session_count,
                COALESCE(SUM(message_count), 0) AS message_count,
                MAX(date) AS latest_activity
            FROM usage_daily_model
            GROUP BY source
            ORDER BY total_tokens DESC, source ASC",
        )
        .map_err(|error| format!("准备读取 token provider 聚合失败: {error}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ProviderTokenUsage {
                source: row.get(0)?,
                total_tokens: row.get::<_, i64>(1)?.max(0) as u64,
                session_count: row.get::<_, i64>(2)?.max(0) as u64,
                message_count: row.get::<_, i64>(3)?.max(0) as u64,
                latest_activity: row.get(4)?,
            })
        })
        .map_err(|error| format!("读取 token provider 聚合失败: {error}"))?;

    collect_rows(rows, "读取 token provider 聚合行失败")
}

fn read_usage_by_day(conn: &Connection) -> Result<Vec<DailyTokenUsage>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                date,
                source,
                COALESCE(SUM(total_tokens), 0) AS total_tokens,
                COALESCE(SUM(session_count), 0) AS session_count,
                COALESCE(SUM(message_count), 0) AS message_count
            FROM usage_daily_model
            GROUP BY date, source
            ORDER BY date ASC, source ASC",
        )
        .map_err(|error| format!("准备读取 daily token 聚合失败: {error}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(DailyTokenUsage {
                date: row.get(0)?,
                source: row.get(1)?,
                total_tokens: row.get::<_, i64>(2)?.max(0) as u64,
                session_count: row.get::<_, i64>(3)?.max(0) as u64,
                message_count: row.get::<_, i64>(4)?.max(0) as u64,
            })
        })
        .map_err(|error| format!("读取 daily token 聚合失败: {error}"))?;

    collect_rows(rows, "读取 daily token 聚合行失败")
}

fn read_usage_by_model(conn: &Connection) -> Result<Vec<ModelTokenUsage>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                source,
                model,
                COALESCE(SUM(total_tokens), 0) AS total_tokens,
                COALESCE(SUM(session_count), 0) AS session_count,
                COALESCE(SUM(message_count), 0) AS message_count
            FROM usage_daily_model
            GROUP BY source, model
            ORDER BY total_tokens DESC, source ASC, model ASC",
        )
        .map_err(|error| format!("准备读取 model token 聚合失败: {error}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ModelTokenUsage {
                source: row.get(0)?,
                model: row.get(1)?,
                total_tokens: row.get::<_, i64>(2)?.max(0) as u64,
                session_count: row.get::<_, i64>(3)?.max(0) as u64,
                message_count: row.get::<_, i64>(4)?.max(0) as u64,
            })
        })
        .map_err(|error| format!("读取 model token 聚合失败: {error}"))?;

    collect_rows(rows, "读取 model token 聚合行失败")
}

fn read_usage_by_model_by_day(conn: &Connection) -> Result<Vec<ModelDailyTokenUsage>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT
                date,
                source,
                model,
                COALESCE(SUM(total_tokens), 0) AS total_tokens,
                COALESCE(SUM(session_count), 0) AS session_count,
                COALESCE(SUM(message_count), 0) AS message_count
            FROM usage_daily_model
            GROUP BY date, source, model
            ORDER BY date ASC, source ASC, total_tokens DESC, model ASC",
        )
        .map_err(|error| format!("准备读取 model daily token 聚合失败: {error}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ModelDailyTokenUsage {
                date: row.get(0)?,
                source: row.get(1)?,
                model: row.get(2)?,
                total_tokens: row.get::<_, i64>(3)?.max(0) as u64,
                session_count: row.get::<_, i64>(4)?.max(0) as u64,
                message_count: row.get::<_, i64>(5)?.max(0) as u64,
            })
        })
        .map_err(|error| format!("读取 model daily token 聚合失败: {error}"))?;

    collect_rows(rows, "读取 model daily token 聚合行失败")
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
    message: &str,
) -> Result<Vec<T>, String> {
    let mut values = Vec::new();
    for row in rows {
        values.push(row.map_err(|error| format!("{message}: {error}"))?);
    }
    Ok(values)
}

fn replace_usage_daily_model_in_conn(
    conn: &mut Connection,
    source: &str,
    rows: Vec<ModelDailyTokenUsage>,
) -> Result<(), String> {
    for row in &rows {
        if row.source != source {
            return Err(format!(
                "model daily token source mismatch: expected {source}, got {}",
                row.source
            ));
        }
    }

    let tx = conn
        .transaction()
        .map_err(|error| format!("打开 model daily token 索引事务失败: {error}"))?;

    tx.execute("DELETE FROM usage_daily_model WHERE source = ?1", [source])
        .map_err(|error| format!("清理 model daily token 索引失败: {error}"))?;

    for row in rows {
        tx.execute(
            "INSERT INTO usage_daily_model (
                source, date, model, total_tokens, session_count, message_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                row.source,
                row.date,
                row.model,
                row.total_tokens as i64,
                row.session_count as i64,
                row.message_count as i64,
            ],
        )
        .map_err(|error| {
            format!(
                "写入 model daily token 索引失败 {}:{}:{}: {error}",
                row.source, row.date, row.model
            )
        })?;
    }

    tx.commit()
        .map_err(|error| format!("提交 model daily token 索引失败: {error}"))
}

fn open_index() -> Result<Connection, String> {
    let path = app_index_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建 session 索引目录失败 {}: {error}", parent.display()))?;
    }

    let conn = Connection::open(&path)
        .map_err(|error| format!("打开 session 索引失败 {}: {error}", path.display()))?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| format!("设置 session 索引 WAL 模式失败: {error}"))?;
    ensure_schema(&conn)?;
    Ok(conn)
}

fn ensure_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS sessions (
            source TEXT NOT NULL,
            id TEXT NOT NULL,
            short_id TEXT NOT NULL,
            title TEXT NOT NULL,
            preview TEXT NOT NULL,
            workspace TEXT NOT NULL,
            model TEXT NOT NULL,
            mode TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at TEXT,
            updated_at TEXT,
            message_count INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL,
            invalid_reason TEXT,
            file_mtime_ms INTEGER,
            file_size INTEGER,
            indexed_at_ms INTEGER NOT NULL,
            deleted_at_ms INTEGER,
            PRIMARY KEY (source, id)
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_source_updated
            ON sessions(source, deleted_at_ms, updated_at);
        CREATE TABLE IF NOT EXISTS source_state (
            source TEXT PRIMARY KEY,
            last_refresh_at_ms INTEGER,
            last_success_at_ms INTEGER,
            last_error TEXT,
            refresh_watermark TEXT
        );
        CREATE TABLE IF NOT EXISTS usage_daily_model (
            source TEXT NOT NULL,
            date TEXT NOT NULL,
            model TEXT NOT NULL,
            total_tokens INTEGER NOT NULL DEFAULT 0,
            session_count INTEGER NOT NULL DEFAULT 0,
            message_count INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (source, date, model)
        );
        CREATE INDEX IF NOT EXISTS idx_usage_daily_model_source_date
            ON usage_daily_model(source, date);",
    )
    .map_err(|error| format!("初始化 session 索引 schema 失败: {error}"))
}

fn upsert_session(
    conn: &Connection,
    record: &SessionRecord,
    indexed_at_ms: i64,
) -> Result<(), String> {
    let (file_mtime_ms, file_size) = file_meta(&record.path);
    conn.execute(
        "INSERT INTO sessions (
            source, id, short_id, title, preview, workspace, model, mode, path,
            created_at, updated_at, message_count, total_tokens, invalid_reason,
            file_mtime_ms, file_size, indexed_at_ms, deleted_at_ms
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, NULL)
        ON CONFLICT(source, id) DO UPDATE SET
            short_id = excluded.short_id,
            title = excluded.title,
            preview = excluded.preview,
            workspace = excluded.workspace,
            model = excluded.model,
            mode = excluded.mode,
            path = excluded.path,
            created_at = excluded.created_at,
            updated_at = excluded.updated_at,
            message_count = excluded.message_count,
            total_tokens = excluded.total_tokens,
            invalid_reason = excluded.invalid_reason,
            file_mtime_ms = excluded.file_mtime_ms,
            file_size = excluded.file_size,
            indexed_at_ms = excluded.indexed_at_ms,
            deleted_at_ms = NULL",
        params![
            record.source,
            record.id,
            record.short_id,
            record.title,
            record.preview,
            record.workspace,
            record.model,
            record.mode,
            record.path,
            record.created_at,
            record.updated_at,
            record.message_count as i64,
            record.total_tokens as i64,
            record.invalid_reason,
            file_mtime_ms,
            file_size,
            indexed_at_ms,
        ],
    )
    .map_err(|error| {
        format!(
            "写入 session 索引失败 {}:{}: {error}",
            record.source, record.id
        )
    })?;
    Ok(())
}

fn file_meta(path: &str) -> (Option<i64>, Option<i64>) {
    let Ok(metadata) = fs::metadata(path) else {
        return (None, None);
    };
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as i64);
    (mtime, Some(metadata.len() as i64))
}

fn read_source_state_from_conn(
    conn: &Connection,
    source: &str,
) -> Result<Option<SourceState>, String> {
    conn.query_row(
        "SELECT source, last_refresh_at_ms, last_success_at_ms, last_error, refresh_watermark
         FROM source_state
         WHERE source = ?1",
        [source],
        |row| {
            Ok(SourceState {
                source: row.get(0)?,
                last_refresh_at_ms: row.get(1)?,
                last_success_at_ms: row.get(2)?,
                last_error: row.get(3)?,
                refresh_watermark: row.get(4)?,
            })
        },
    )
    .optional()
    .map_err(|error| format!("读取 source_state 失败: {error}"))
}

fn upsert_source_state(conn: &Connection, state: &SourceState) -> Result<(), String> {
    conn.execute(
        "INSERT INTO source_state (
            source, last_refresh_at_ms, last_success_at_ms, last_error, refresh_watermark
        ) VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(source) DO UPDATE SET
            last_refresh_at_ms = excluded.last_refresh_at_ms,
            last_success_at_ms = excluded.last_success_at_ms,
            last_error = excluded.last_error,
            refresh_watermark = excluded.refresh_watermark",
        params![
            state.source,
            state.last_refresh_at_ms,
            state.last_success_at_ms,
            state.last_error,
            state.refresh_watermark,
        ],
    )
    .map_err(|error| format!("写入 source_state 失败: {error}"))?;
    Ok(())
}

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_usage_readers_aggregate_usage_daily_model_and_ignore_sessions() {
        let conn = Connection::open_in_memory().expect("open in-memory index");
        ensure_schema(&conn).expect("create schema");

        conn.execute(
            "INSERT INTO sessions (
                source, id, short_id, title, preview, workspace, model, mode, path,
                created_at, updated_at, message_count, total_tokens, invalid_reason,
                file_mtime_ms, file_size, indexed_at_ms, deleted_at_ms
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, NULL, NULL, NULL, ?14, NULL)",
            params![
                "claude",
                "session-row-ignored",
                "ignored",
                "Ignored session row",
                "ignored",
                "workspace",
                "session-model",
                "default",
                "/tmp/ignored.jsonl",
                "2026-05-10T00:00:00Z",
                "2026-05-10T01:00:00Z",
                99_i64,
                9_999_i64,
                1_i64,
            ],
        )
        .expect("insert ignored session usage");

        conn.execute(
            "INSERT INTO usage_daily_model (
                source, date, model, total_tokens, session_count, message_count
            ) VALUES
                ('claude', '2026-05-11', 'opus', 300, 3, 30),
                ('claude', '2026-05-11', 'sonnet', 500, 5, 50),
                ('claude', '2026-05-12', 'sonnet', 200, 2, 20),
                ('codex', '2026-05-11', 'gpt', 400, 4, 40)",
            [],
        )
        .expect("insert model daily usage");

        let provider_rows = read_usage_by_provider(&conn)
            .expect("read provider usage")
            .into_iter()
            .map(|row| {
                (
                    row.source,
                    row.total_tokens,
                    row.session_count,
                    row.message_count,
                    row.latest_activity,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            provider_rows,
            vec![
                (
                    "claude".to_string(),
                    1_000,
                    10,
                    100,
                    Some("2026-05-12".to_string()),
                ),
                (
                    "codex".to_string(),
                    400,
                    4,
                    40,
                    Some("2026-05-11".to_string()),
                ),
            ]
        );

        let daily_rows = read_usage_by_day(&conn)
            .expect("read daily usage")
            .into_iter()
            .map(|row| {
                (
                    row.date,
                    row.source,
                    row.total_tokens,
                    row.session_count,
                    row.message_count,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            daily_rows,
            vec![
                ("2026-05-11".to_string(), "claude".to_string(), 800, 8, 80),
                ("2026-05-11".to_string(), "codex".to_string(), 400, 4, 40),
                ("2026-05-12".to_string(), "claude".to_string(), 200, 2, 20),
            ]
        );

        let model_rows = read_usage_by_model(&conn)
            .expect("read model usage")
            .into_iter()
            .map(|row| {
                (
                    row.source,
                    row.model,
                    row.total_tokens,
                    row.session_count,
                    row.message_count,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            model_rows,
            vec![
                ("claude".to_string(), "sonnet".to_string(), 700, 7, 70),
                ("codex".to_string(), "gpt".to_string(), 400, 4, 40),
                ("claude".to_string(), "opus".to_string(), 300, 3, 30),
            ]
        );

        let model_daily_rows = read_usage_by_model_by_day(&conn)
            .expect("read model daily usage")
            .into_iter()
            .map(|row| {
                (
                    row.date,
                    row.source,
                    row.model,
                    row.total_tokens,
                    row.session_count,
                    row.message_count,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            model_daily_rows,
            vec![
                (
                    "2026-05-11".to_string(),
                    "claude".to_string(),
                    "sonnet".to_string(),
                    500,
                    5,
                    50,
                ),
                (
                    "2026-05-11".to_string(),
                    "claude".to_string(),
                    "opus".to_string(),
                    300,
                    3,
                    30,
                ),
                (
                    "2026-05-11".to_string(),
                    "codex".to_string(),
                    "gpt".to_string(),
                    400,
                    4,
                    40,
                ),
                (
                    "2026-05-12".to_string(),
                    "claude".to_string(),
                    "sonnet".to_string(),
                    200,
                    2,
                    20,
                ),
            ]
        );
    }

    #[test]
    fn replace_usage_daily_model_rejects_mismatched_row_source_without_changes() {
        let mut conn = Connection::open_in_memory().expect("open in-memory index");
        ensure_schema(&conn).expect("create schema");

        replace_usage_daily_model_in_conn(
            &mut conn,
            "claude",
            vec![ModelDailyTokenUsage {
                date: "2026-05-12".to_string(),
                source: "claude".to_string(),
                model: "sonnet".to_string(),
                total_tokens: 100,
                session_count: 2,
                message_count: 7,
            }],
        )
        .expect("insert claude usage");

        let error = replace_usage_daily_model_in_conn(
            &mut conn,
            "claude",
            vec![ModelDailyTokenUsage {
                date: "2026-05-13".to_string(),
                source: "codex".to_string(),
                model: "gpt".to_string(),
                total_tokens: 50,
                session_count: 1,
                message_count: 3,
            }],
        )
        .expect_err("reject mismatched source");
        assert!(error.contains("source"));

        let rows = conn
            .prepare(
                "SELECT source, date, model, total_tokens, session_count, message_count
                 FROM usage_daily_model
                 ORDER BY source, date, model",
            )
            .expect("prepare usage query")
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .expect("query usage rows")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("collect usage rows");

        assert_eq!(
            rows,
            vec![(
                "claude".to_string(),
                "2026-05-12".to_string(),
                "sonnet".to_string(),
                100,
                2,
                7,
            ),]
        );
    }

    #[test]
    fn replace_usage_daily_model_replaces_only_matching_source() {
        let mut conn = Connection::open_in_memory().expect("open in-memory index");
        ensure_schema(&conn).expect("create schema");

        replace_usage_daily_model_in_conn(
            &mut conn,
            "claude",
            vec![ModelDailyTokenUsage {
                date: "2026-05-12".to_string(),
                source: "claude".to_string(),
                model: "sonnet".to_string(),
                total_tokens: 100,
                session_count: 2,
                message_count: 7,
            }],
        )
        .expect("insert claude usage");
        replace_usage_daily_model_in_conn(
            &mut conn,
            "codex",
            vec![ModelDailyTokenUsage {
                date: "2026-05-12".to_string(),
                source: "codex".to_string(),
                model: "gpt".to_string(),
                total_tokens: 50,
                session_count: 1,
                message_count: 3,
            }],
        )
        .expect("insert codex usage");
        replace_usage_daily_model_in_conn(
            &mut conn,
            "claude",
            vec![ModelDailyTokenUsage {
                date: "2026-05-13".to_string(),
                source: "claude".to_string(),
                model: "opus".to_string(),
                total_tokens: 200,
                session_count: 4,
                message_count: 9,
            }],
        )
        .expect("replace claude usage");

        let rows = conn
            .prepare(
                "SELECT source, date, model, total_tokens, session_count, message_count
                 FROM usage_daily_model
                 ORDER BY source, date, model",
            )
            .expect("prepare usage query")
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            })
            .expect("query usage rows")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("collect usage rows");

        assert_eq!(
            rows,
            vec![
                (
                    "claude".to_string(),
                    "2026-05-13".to_string(),
                    "opus".to_string(),
                    200,
                    4,
                    9,
                ),
                (
                    "codex".to_string(),
                    "2026-05-12".to_string(),
                    "gpt".to_string(),
                    50,
                    1,
                    3,
                ),
            ]
        );
    }
}
