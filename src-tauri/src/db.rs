use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::Path;
use crate::models::*;
use crate::error::AppResult;

pub fn init_db(path: &Path) -> SqliteResult<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version TEXT PRIMARY KEY NOT NULL,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    let already_applied: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE version = '001'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !already_applied {
        let sql = include_str!("migrations/001_initial.sql");
        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO _migrations (version, applied_at) VALUES ('001', datetime('now'))",
            [],
        )?;
    }

    let migration_002_applied: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE version = '002'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !migration_002_applied {
        // Safely alter table to add name column. Catch error if it already exists.
        let _ = conn.execute("ALTER TABLE version_snapshots ADD COLUMN name TEXT", []);
        conn.execute(
            "INSERT INTO _migrations (version, applied_at) VALUES ('002', datetime('now'))",
            [],
        )?;
    }

    let migration_003_applied: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE version = '003'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !migration_003_applied {
        let sql = "
            CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY NOT NULL,
                provider_type TEXT NOT NULL,
                name TEXT NOT NULL,
                use_case TEXT NOT NULL DEFAULT '',
                icon TEXT NOT NULL DEFAULT '🤖',
                is_custom INTEGER NOT NULL DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
        ";
        conn.execute_batch(sql)?;

        let defaults = vec![
            ("openai/gpt-4o-mini", "openrouter", "GPT-4o Mini", "Fast proofreading, grammar & syntax polishing", "⚡"),
            ("meta-llama/llama-3.3-70b-instruct", "openrouter", "Llama 3.3 70B", "General prose editing & standard revision", "🦙"),
            ("z-ai/glm-4.5-air", "openrouter", "GLM 4.5 Air", "Creative novels, character dialogue & SEO", "🎨"),
            ("openrouter/owl-alpha", "openrouter", "Owl Alpha", "Heavy content synthesis & professional drafts", "🦉"),
            ("google/gemma-4-31b-it", "openrouter", "Gemma 4 31B", "Grammar review & high-accuracy translation", "💎"),
            ("openai/gpt-oss-120b", "openrouter", "GPT-OSS 120B", "Academic papers, logic flow & structured essays", "🎓"),
            ("deepseek/deepseek-v4-flash", "openrouter", "DeepSeek V4 Flash", "Long-form narrative & fast token translation", "🌀"),
            
            ("openai/gpt-oss-120b", "groq", "GPT-OSS 120B", "Academic papers, logic flow & structured essays", "🎓"),
            ("llama-3.1-8b-instant", "groq", "Llama 3.1 8B Instant", "Ultra-fast grammar corrections & simple revisions", "⚡"),
            ("llama-3.3-70b-versatile", "groq", "Llama 3.3 70B Versatile", "General prose editing & standard revision", "🦙"),
            ("meta-llama/llama-4-scout-17b-16e-instruct", "groq", "Llama 4 Scout 17B (16e)", "Highly creative, nuance & character dialog", "🔍"),
            
            ("nvidia/nemotron-3-nano-omni-30b-a3b-reasoning", "nvidia", "Nemotron 3 Nano Omni", "Multi-modal reasoning & long-context text synthesis", "🧠"),
            ("deepseek-ai/deepseek-v4-flash", "nvidia", "DeepSeek V4 Flash", "MoE optimized for ultra-fast coding & drafts", "🌀"),
            ("deepseek-ai/deepseek-v4-pro", "nvidia", "DeepSeek V4 Pro", "High MoE scaling for heavy editorial revisions", "🚀"),
            ("mistralai/mistral-medium-3.5-128b", "nvidia", "Mistral Medium 3.5", "High performing agentic rewrite", "🌪️"),
            ("z-ai/glm-5.1", "nvidia", "GLM 5.1", "Flagship LLM for agentic workflows & deep reasoning", "✨"),
            ("moonshotai/kimi-k2.6", "nvidia", "Kimi k2.6", "1T MoE long-context coding & reasoning", "🌊")
        ];

        for (id, provider_type, name, use_case, icon) in defaults {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO models (id, provider_type, name, use_case, icon, is_custom, enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, datetime('now'), datetime('now'))",
                params![id, provider_type, name, use_case, icon],
            );
        }

        conn.execute(
            "INSERT INTO _migrations (version, applied_at) VALUES ('003', datetime('now'))",
            [],
        )?;
    }

    let migration_004_applied: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE version = '004'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !migration_004_applied {
        // Safely add enabled column to models table for existing databases.
        let _ = conn.execute("ALTER TABLE models ADD COLUMN enabled INTEGER NOT NULL DEFAULT 1", []);
        conn.execute(
            "INSERT INTO _migrations (version, applied_at) VALUES ('004', datetime('now'))",
            [],
        )?;
    }

    Ok(())
}

// ─── Page CRUD ──────────────────────────────────────────────────────────

pub fn insert_page(conn: &Connection, id: &str, title: &str, content: &str) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO pages (id, title, content, format_type, created_at, updated_at, local_status)
         VALUES (?1, ?2, ?3, 'markdown', datetime('now'), datetime('now'), 'active')",
        params![id, title, content],
    )?;
    Ok(())
}

pub fn list_pages(conn: &Connection) -> SqliteResult<Vec<Page>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, format_type, created_at, updated_at,
                current_version_id, last_review_session_id, local_status
         FROM pages WHERE local_status = 'active' ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Page {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            format_type: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            current_version_id: row.get(6)?,
            last_review_session_id: row.get(7)?,
            local_status: row.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn get_page(conn: &Connection, id: &str) -> SqliteResult<Option<Page>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, content, format_type, created_at, updated_at,
                current_version_id, last_review_session_id, local_status
         FROM pages WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Page {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            format_type: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
            current_version_id: row.get(6)?,
            last_review_session_id: row.get(7)?,
            local_status: row.get(8)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn update_page_content(conn: &Connection, id: &str, title: &str, content: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE pages SET title = ?2, content = ?3, updated_at = datetime('now') WHERE id = ?1",
        params![id, title, content],
    )?;
    Ok(())
}

pub fn rename_page(conn: &Connection, id: &str, title: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE pages SET title = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![id, title],
    )?;
    Ok(())
}

pub fn delete_page(conn: &Connection, id: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE pages SET local_status = 'deleted', updated_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

pub fn set_page_review_session(conn: &Connection, page_id: &str, session_id: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE pages SET last_review_session_id = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![page_id, session_id],
    )?;
    Ok(())
}

pub fn clear_page_review_session(conn: &Connection, page_id: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE pages SET last_review_session_id = NULL, updated_at = datetime('now') WHERE id = ?1",
        params![page_id],
    )?;
    Ok(())
}

pub fn set_page_version(conn: &Connection, page_id: &str, version_id: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE pages SET current_version_id = ?2, updated_at = datetime('now') WHERE id = ?1",
        params![page_id, version_id],
    )?;
    Ok(())
}

// ─── Review Session CRUD ────────────────────────────────────────────────

pub fn insert_review_session(conn: &Connection, session: &ReviewSession) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO review_sessions (id, page_id, provider_id, model_id, prompt_version, raw_response, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            session.id, session.page_id, session.provider_id, session.model_id,
            session.prompt_version, session.raw_response, session.status, session.created_at
        ],
    )?;
    Ok(())
}

pub fn update_review_session_response(conn: &Connection, id: &str, raw_response: &str, status: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE review_sessions SET raw_response = ?2, status = ?3 WHERE id = ?1",
        params![id, raw_response, status],
    )?;
    Ok(())
}

// ─── Suggestion CRUD ────────────────────────────────────────────────────

pub fn insert_suggestion_group(conn: &Connection, group: &SuggestionGroup) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO suggestion_groups (id, page_id, review_session_id, category, label, approval_state)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            group.id, group.page_id, group.review_session_id,
            group.category, group.label, group.approval_state
        ],
    )?;
    Ok(())
}

pub fn insert_suggestion_item(conn: &Connection, item: &SuggestionItem) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO suggestion_items (id, group_id, original_text, replacement_text, span_start, span_end, explanation, confidence, approval_state, conflict_state)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            item.id, item.group_id, item.original_text, item.replacement_text,
            item.span_start as i64, item.span_end as i64, item.explanation,
            item.confidence, item.approval_state, item.conflict_state
        ],
    )?;
    Ok(())
}

pub fn get_suggestion_groups_with_items(conn: &Connection, session_id: &str) -> SqliteResult<Vec<SuggestionGroupWithItems>> {
    let mut group_stmt = conn.prepare(
        "SELECT id, page_id, review_session_id, category, label, approval_state
         FROM suggestion_groups WHERE review_session_id = ?1 ORDER BY category",
    )?;
    let groups: Vec<SuggestionGroup> = group_stmt.query_map(params![session_id], |row| {
        Ok(SuggestionGroup {
            id: row.get(0)?,
            page_id: row.get(1)?,
            review_session_id: row.get(2)?,
            category: row.get(3)?,
            label: row.get(4)?,
            approval_state: row.get(5)?,
        })
    })?.collect::<SqliteResult<Vec<_>>>()?;

    let mut result = Vec::new();
    for group in groups {
        let mut item_stmt = conn.prepare(
            "SELECT id, group_id, original_text, replacement_text, span_start, span_end,
                    explanation, confidence, approval_state, conflict_state
             FROM suggestion_items WHERE group_id = ?1 ORDER BY span_start",
        )?;
        let items: Vec<SuggestionItem> = item_stmt.query_map(params![group.id], |row| {
            let span_start: i64 = row.get(4)?;
            let span_end: i64 = row.get(5)?;
            Ok(SuggestionItem {
                id: row.get(0)?,
                group_id: row.get(1)?,
                original_text: row.get(2)?,
                replacement_text: row.get(3)?,
                span_start: span_start as usize,
                span_end: span_end as usize,
                explanation: row.get(6)?,
                confidence: row.get(7)?,
                approval_state: row.get(8)?,
                conflict_state: row.get(9)?,
            })
        })?.collect::<SqliteResult<Vec<_>>>()?;

        result.push(SuggestionGroupWithItems { group, items });
    }
    Ok(result)
}

pub fn update_item_approval(conn: &Connection, item_id: &str, state: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE suggestion_items SET approval_state = ?2 WHERE id = ?1",
        params![item_id, state],
    )?;
    Ok(())
}

pub fn update_group_approval(conn: &Connection, group_id: &str, state: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE suggestion_groups SET approval_state = ?2 WHERE id = ?1",
        params![group_id, state],
    )?;
    // Also update all items in the group
    conn.execute(
        "UPDATE suggestion_items SET approval_state = ?2 WHERE group_id = ?1",
        params![group_id, state],
    )?;
    Ok(())
}

pub fn get_approved_items_for_session(conn: &Connection, session_id: &str) -> SqliteResult<Vec<SuggestionItem>> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.group_id, si.original_text, si.replacement_text,
                si.span_start, si.span_end, si.explanation, si.confidence,
                si.approval_state, si.conflict_state
         FROM suggestion_items si
         JOIN suggestion_groups sg ON si.group_id = sg.id
         WHERE sg.review_session_id = ?1 AND si.approval_state = 'approved'
         ORDER BY si.span_start",
    )?;
    let items = stmt.query_map(params![session_id], |row| {
        let span_start: i64 = row.get(4)?;
        let span_end: i64 = row.get(5)?;
        Ok(SuggestionItem {
            id: row.get(0)?,
            group_id: row.get(1)?,
            original_text: row.get(2)?,
            replacement_text: row.get(3)?,
            span_start: span_start as usize,
            span_end: span_end as usize,
            explanation: row.get(6)?,
            confidence: row.get(7)?,
            approval_state: row.get(8)?,
            conflict_state: row.get(9)?,
        })
    })?.collect::<SqliteResult<Vec<_>>>()?;
    Ok(items)
}

pub fn get_all_items_for_session(conn: &Connection, session_id: &str) -> SqliteResult<Vec<SuggestionItem>> {
    let mut stmt = conn.prepare(
        "SELECT si.id, si.group_id, si.original_text, si.replacement_text,
                si.span_start, si.span_end, si.explanation, si.confidence,
                si.approval_state, si.conflict_state
         FROM suggestion_items si
         JOIN suggestion_groups sg ON si.group_id = sg.id
         WHERE sg.review_session_id = ?1
         ORDER BY si.span_start",
    )?;
    let items = stmt.query_map(params![session_id], |row| {
        let span_start: i64 = row.get(4)?;
        let span_end: i64 = row.get(5)?;
        Ok(SuggestionItem {
            id: row.get(0)?,
            group_id: row.get(1)?,
            original_text: row.get(2)?,
            replacement_text: row.get(3)?,
            span_start: span_start as usize,
            span_end: span_end as usize,
            explanation: row.get(6)?,
            confidence: row.get(7)?,
            approval_state: row.get(8)?,
            conflict_state: row.get(9)?,
        })
    })?.collect::<SqliteResult<Vec<_>>>()?;
    Ok(items)
}

// ─── Version Snapshots ──────────────────────────────────────────────────

pub fn insert_version_snapshot(conn: &Connection, snapshot: &VersionSnapshot) -> SqliteResult<()> {
    let ids_json = serde_json::to_string(&snapshot.applied_suggestion_ids).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO version_snapshots (id, page_id, content, source_review_session_id, applied_suggestion_ids, created_at, name)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            snapshot.id, snapshot.page_id, snapshot.content,
            snapshot.source_review_session_id, ids_json, snapshot.created_at, snapshot.name
        ],
    )?;
    Ok(())
}

pub fn list_versions(conn: &Connection, page_id: &str) -> SqliteResult<Vec<VersionSnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, page_id, content, source_review_session_id, applied_suggestion_ids, created_at, name
         FROM version_snapshots WHERE page_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![page_id], |row| {
        let ids_str: String = row.get(4)?;
        let applied_ids: Vec<String> = serde_json::from_str(&ids_str).unwrap_or_default();
        Ok(VersionSnapshot {
            id: row.get(0)?,
            page_id: row.get(1)?,
            content: row.get(2)?,
            source_review_session_id: row.get(3)?,
            applied_suggestion_ids: applied_ids,
            created_at: row.get(5)?,
            name: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn get_version(conn: &Connection, version_id: &str) -> SqliteResult<Option<VersionSnapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, page_id, content, source_review_session_id, applied_suggestion_ids, created_at, name
         FROM version_snapshots WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![version_id], |row| {
        let ids_str: String = row.get(4)?;
        let applied_ids: Vec<String> = serde_json::from_str(&ids_str).unwrap_or_default();
        Ok(VersionSnapshot {
            id: row.get(0)?,
            page_id: row.get(1)?,
            content: row.get(2)?,
            source_review_session_id: row.get(3)?,
            applied_suggestion_ids: applied_ids,
            created_at: row.get(5)?,
            name: row.get(6)?,
        })
    })?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn rename_version(conn: &Connection, version_id: &str, name: &str) -> SqliteResult<()> {
    conn.execute(
        "UPDATE version_snapshots SET name = ?2 WHERE id = ?1",
        params![version_id, name],
    )?;
    Ok(())
}

pub fn delete_version(conn: &Connection, version_id: &str) -> SqliteResult<()> {
    conn.execute(
        "DELETE FROM version_snapshots WHERE id = ?1",
        params![version_id],
    )?;
    Ok(())
}


// ─── Provider Config CRUD ───────────────────────────────────────────────

pub fn insert_provider_config(conn: &Connection, config: &ProviderConfig) -> SqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO provider_configs (id, name, provider_type, endpoint, selected_model, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            config.id, config.name, config.provider_type, config.endpoint,
            config.selected_model, config.enabled, config.created_at, config.updated_at
        ],
    )?;
    Ok(())
}

pub fn list_provider_configs(conn: &Connection) -> SqliteResult<Vec<ProviderConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, provider_type, endpoint, selected_model, enabled, created_at, updated_at
         FROM provider_configs ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ProviderConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            provider_type: row.get(2)?,
            endpoint: row.get(3)?,
            selected_model: row.get(4)?,
            enabled: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    rows.collect()
}

pub fn delete_provider_config(conn: &Connection, id: &str) -> SqliteResult<()> {
    conn.execute("DELETE FROM provider_configs WHERE id = ?1", params![id])?;
    Ok(())
}

// ─── Model Config CRUD ──────────────────────────────────────────────────

pub fn insert_model_config(conn: &Connection, config: &ModelConfig) -> SqliteResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO models (id, provider_type, name, use_case, icon, is_custom, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            config.id, config.provider_type, config.name, config.use_case,
            config.icon, if config.is_custom { 1 } else { 0 },
            if config.enabled { 1 } else { 0 },
            config.created_at, config.updated_at
        ],
    )?;
    Ok(())
}

pub fn list_model_configs(conn: &Connection) -> SqliteResult<Vec<ModelConfig>> {
    let mut stmt = conn.prepare(
        "SELECT id, provider_type, name, use_case, icon, is_custom, enabled, created_at, updated_at
         FROM models ORDER BY name",
    )?;
    let rows = stmt.query_map([], |row| {
        let is_custom_val: i32 = row.get(5)?;
        let enabled_val: i32 = row.get(6)?;
        Ok(ModelConfig {
            id: row.get(0)?,
            provider_type: row.get(1)?,
            name: row.get(2)?,
            use_case: row.get(3)?,
            icon: row.get(4)?,
            is_custom: is_custom_val != 0,
            enabled: enabled_val != 0,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn delete_model_config(conn: &Connection, id: &str) -> SqliteResult<()> {
    conn.execute("DELETE FROM models WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn reset_defaults(conn: &Connection) -> SqliteResult<Vec<ModelConfig>> {
    // Delete only non-custom models
    conn.execute("DELETE FROM models WHERE is_custom = 0", [])?;

    let defaults = vec![
        ("openai/gpt-4o-mini", "openrouter", "GPT-4o Mini", "Fast proofreading, grammar & syntax polishing", "⚡"),
        ("meta-llama/llama-3.3-70b-instruct", "openrouter", "Llama 3.3 70B", "General prose editing & standard revision", "🦙"),
        ("z-ai/glm-4.5-air", "openrouter", "GLM 4.5 Air", "Creative novels, character dialogue & SEO", "🎨"),
        ("openrouter/owl-alpha", "openrouter", "Owl Alpha", "Heavy content synthesis & professional drafts", "🦉"),
        ("google/gemma-4-31b-it", "openrouter", "Gemma 4 31B", "Grammar review & high-accuracy translation", "💎"),
        ("openai/gpt-oss-120b", "openrouter", "GPT-OSS 120B", "Academic papers, logic flow & structured essays", "🎓"),
        ("deepseek/deepseek-v4-flash", "openrouter", "DeepSeek V4 Flash", "Long-form narrative & fast token translation", "🌀"),
        
        ("openai/gpt-oss-120b", "groq", "GPT-OSS 120B", "Academic papers, logic flow & structured essays", "🎓"),
        ("llama-3.1-8b-instant", "groq", "Llama 3.1 8B Instant", "Ultra-fast grammar corrections & simple revisions", "⚡"),
        ("llama-3.3-70b-versatile", "groq", "Llama 3.3 70B Versatile", "General prose editing & standard revision", "🦙"),
        ("meta-llama/llama-4-scout-17b-16e-instruct", "groq", "Llama 4 Scout 17B (16e)", "Highly creative, nuance & character dialog", "🔍"),
        
        ("nvidia/nemotron-3-nano-omni-30b-a3b-reasoning", "nvidia", "Nemotron 3 Nano Omni", "Multi-modal reasoning & long-context text synthesis", "🧠"),
        ("deepseek-ai/deepseek-v4-flash", "nvidia", "DeepSeek V4 Flash", "MoE optimized for ultra-fast coding & drafts", "🌀"),
        ("deepseek-ai/deepseek-v4-pro", "nvidia", "DeepSeek V4 Pro", "High MoE scaling for heavy editorial revisions", "🚀"),
        ("mistralai/mistral-medium-3.5-128b", "nvidia", "Mistral Medium 3.5", "High performing agentic rewrite", "🌪️"),
        ("z-ai/glm-5.1", "nvidia", "GLM 5.1", "Flagship LLM for agentic workflows & deep reasoning", "✨"),
        ("moonshotai/kimi-k2.6", "nvidia", "Kimi k2.6", "1T MoE long-context coding & reasoning", "🌊")
    ];

    for (id, provider_type, name, use_case, icon) in defaults {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO models (id, provider_type, name, use_case, icon, is_custom, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, datetime('now'), datetime('now'))",
            params![id, provider_type, name, use_case, icon],
        );
    }

    list_model_configs(conn)
}

// ─── App State ──────────────────────────────────────────────────────────

pub fn get_app_state(conn: &Connection, key: &str) -> SqliteResult<Option<String>> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_state (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
        [],
    )?;
    let mut stmt = conn.prepare("SELECT value FROM app_state WHERE key = ?1")?;
    let mut rows = stmt.query_map(params![key], |row| row.get(0))?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn set_app_state(conn: &Connection, key: &str, value: &str) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_state (key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}
