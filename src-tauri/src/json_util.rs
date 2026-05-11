//! Provides JSON traversal and whitespace compaction helpers for parsers.

use serde_json::Value;

pub fn string_at(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

pub fn number_at(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

pub fn compact(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn content_to_text(content: Option<&Value>) -> String {
    match content {
        Some(Value::String(text)) => compact(text),
        Some(Value::Array(items)) => compact(
            &items
                .iter()
                .filter_map(|item| {
                    if let Some(text) = item.as_str() {
                        return Some(text.to_string());
                    }
                    if item.get("type").and_then(Value::as_str) == Some("text") {
                        return item.get("text").and_then(Value::as_str).map(str::to_string);
                    }
                    item.get("content")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
                .join(" "),
        ),
        Some(Value::Object(map)) => map
            .get("content")
            .and_then(Value::as_str)
            .map(compact)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn string_at_reads_string_values() {
        assert_eq!(
            string_at(&json!({ "name": "cap" }), "name"),
            Some("cap".to_string())
        );
    }

    #[test]
    fn number_at_reads_u64_values() {
        assert_eq!(number_at(&json!({ "count": 3 }), "count"), Some(3));
    }

    #[test]
    fn compact_collapses_whitespace() {
        assert_eq!(compact("a\n  b\t c"), "a b c");
    }

    #[test]
    fn content_to_text_extracts_text_arrays() {
        let value = json!([{ "type": "text", "text": "hello" }, { "content": "world" }]);
        assert_eq!(content_to_text(Some(&value)), "hello world");
    }
}
