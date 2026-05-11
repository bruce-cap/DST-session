//! Maintains the app-owned SQLite session index.

use crate::model::{RefreshResult, SessionRecord, SourceState};
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
        );",
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

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
