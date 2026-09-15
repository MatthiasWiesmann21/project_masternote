use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::models::{Category, Contact, Coworker, Note, NoteLink, NoteStatistics, NoteTemplate, Reminder, SavedSearch, SearchResult, Tag, CategoryCount};

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();

        // Step 1: Create schema_version table (migration 0)
        let sql0 = include_str!("../migrations/0000_schema_version.sql");
        conn.execute_batch(sql0)?;

        // Step 2: Check if this is a legacy DB (migrated via old method, no version recorded)
        let has_notes_table: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='notes')",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if has_notes_table {
            // Legacy DB — check if any version has been recorded
            let current_version: i64 = conn
                .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |row| {
                    row.get(0)
                })
                .unwrap_or(0);

            if current_version == 0 {
                // Old method was used; mark v1, v2, v3 as already applied
                conn.execute(
                    "INSERT OR IGNORE INTO schema_version (version) VALUES (1), (2), (3)",
                    [],
                )?;
            }
        }

        // Step 3: Run pending migrations in order
        let migrations: &[(i64, &str)] = &[
            (1, include_str!("../migrations/0001_init.sql")),
            (2, include_str!("../migrations/0002_contacts_coworkers.sql")),
            (3, include_str!("../migrations/0003_contact_phones.sql")),
            (4, include_str!("../migrations/0004_recurring_reminders.sql")),
            (5, include_str!("../migrations/0005_note_archiving.sql")),
            (6, include_str!("../migrations/0006_note_sort_order.sql")),
            (7, include_str!("../migrations/0007_note_templates.sql")),
            (8, include_str!("../migrations/0008_note_links.sql")),
            (9, include_str!("../migrations/0009_saved_searches.sql")),
            (10, include_str!("../migrations/0010_note_pinning.sql")),
        ];

        let current_version: i64 = conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        for (version, sql) in migrations {
            if *version > current_version {
                log::info!("Running migration v{}", version);
                conn.execute_batch(sql)?;
                conn.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    params![version],
                )?;
            }
        }

        Ok(())
    }

    pub fn get_schema_version(&self) -> Result<i64, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
    }

    fn now_iso() -> String {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    // --- Internal helpers that take a &Connection (no locking) ---

    fn fetch_tags_for_note(conn: &Connection, note_id: i64) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name FROM tags t
             JOIN note_tags nt ON nt.tag_id = t.id
             WHERE nt.note_id = ?1
             ORDER BY t.name",
        )?;
        let tags = stmt
            .query_map(params![note_id], |row| Ok(Tag { id: row.get(0)?, name: row.get(1)? }))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    fn fetch_reminder_for_note(
        conn: &Connection,
        note_id: i64,
    ) -> Result<Option<Reminder>, rusqlite::Error> {
        let reminder = conn
            .query_row(
                "SELECT id, note_id, due_at, fired, calendar_event_id, recur_interval, recur_unit FROM reminders WHERE note_id = ?1",
                params![note_id],
                |row| {
                    Ok(Reminder {
                        id: row.get(0)?,
                        note_id: row.get(1)?,
                        due_at: row.get(2)?,
                        fired: row.get(3)?,
                        calendar_event_id: row.get(4)?,
                        recur_interval: row.get(5)?,
                        recur_unit: row.get(6)?,
                    })
                },
            )
            .optional()?;
        Ok(reminder)
    }

    fn fetch_contact(conn: &Connection, contact_id: Option<i64>) -> Result<Option<Contact>, rusqlite::Error> {
        if let Some(cid) = contact_id {
            conn.query_row(
                "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, phone, mobile, created_at, updated_at
                 FROM contacts WHERE id = ?1",
                params![cid],
                Self::row_to_contact,
            )
            .optional()
        } else {
            Ok(None)
        }
    }

    fn fetch_coworker(conn: &Connection, coworker_id: Option<i64>) -> Result<Option<Coworker>, rusqlite::Error> {
        if let Some(wid) = coworker_id {
            conn.query_row(
                "SELECT id, last_name, first_name, email, created_at, updated_at
                 FROM coworkers WHERE id = ?1",
                params![wid],
                Self::row_to_coworker,
            )
            .optional()
        } else {
            Ok(None)
        }
    }

    fn row_to_contact(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
        Ok(Contact {
            id: row.get(0)?,
            last_name: row.get(1)?,
            first_name: row.get(2)?,
            address: row.get(3)?,
            email: row.get(4)?,
            gender: row.get(5)?,
            kind: row.get(6)?,
            company_name: row.get(7)?,
            customer_identifier: row.get(8)?,
            phone: row.get(9)?,
            mobile: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }

    fn row_to_coworker(row: &rusqlite::Row) -> rusqlite::Result<Coworker> {
        Ok(Coworker {
            id: row.get(0)?,
            last_name: row.get(1)?,
            first_name: row.get(2)?,
            email: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }

    fn fetch_note_row(row: &rusqlite::Row) -> rusqlite::Result<Note> {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            category_id: row.get(3)?,
            category_name: row.get(4)?,
            category_color: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            source: row.get(8)?,
            tags: vec![],
            reminder: None,
            contact_id: row.get(9)?,
            coworker_id: row.get(10)?,
            contact: None,
            coworker: None,
            archived: row.get(11)?,
            pinned: row.get(13)?,
            sort_order: row.get(12)?,
            links: vec![],
            backlinks: vec![],
        })
    }

    const NOTE_SELECT: &'static str =
        "SELECT n.id, n.title, n.content, n.category_id, c.name, c.color, \
         n.created_at, n.updated_at, n.source, n.contact_id, n.coworker_id, \
         n.archived, n.sort_order, n.pinned \
         FROM notes n \
         LEFT JOIN categories c ON n.category_id = c.id";

    fn fetch_note_links(conn: &Connection, note_id: i64) -> Result<Vec<NoteLink>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title FROM note_links nl
             JOIN notes n ON n.id = nl.to_note_id
             WHERE nl.from_note_id = ?1 ORDER BY n.title",
        )?;
        let links = stmt
            .query_map(params![note_id], |row| Ok(NoteLink { id: row.get(0)?, title: row.get(1)? }))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(links)
    }

    fn fetch_note_backlinks(conn: &Connection, note_id: i64) -> Result<Vec<NoteLink>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title FROM note_links nl
             JOIN notes n ON n.id = nl.from_note_id
             WHERE nl.to_note_id = ?1 ORDER BY n.title",
        )?;
        let links = stmt
            .query_map(params![note_id], |row| Ok(NoteLink { id: row.get(0)?, title: row.get(1)? }))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(links)
    }

    fn enrich_note(conn: &Connection, note: &mut Note) -> Result<(), rusqlite::Error> {
        note.tags = Self::fetch_tags_for_note(conn, note.id)?;
        note.reminder = Self::fetch_reminder_for_note(conn, note.id)?;
        note.contact = Self::fetch_contact(conn, note.contact_id)?;
        note.coworker = Self::fetch_coworker(conn, note.coworker_id)?;
        note.links = Self::fetch_note_links(conn, note.id)?;
        note.backlinks = Self::fetch_note_backlinks(conn, note.id)?;
        Ok(())
    }

    // --- Note CRUD ---

    pub fn create_note(
        &self,
        title: &str,
        content: &str,
        category_id: Option<i64>,
    ) -> Result<Note, rusqlite::Error> {
        self.create_note_full(title, content, category_id, None, None)
    }

    pub fn create_note_full(
        &self,
        title: &str,
        content: &str,
        category_id: Option<i64>,
        contact_id: Option<i64>,
        coworker_id: Option<i64>,
    ) -> Result<Note, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notes (title, content, category_id, contact_id, coworker_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![title, content, category_id, contact_id, coworker_id, now],
        )?;
        let id = conn.last_insert_rowid();

        let mut note = conn
            .query_row(
                &format!("{} WHERE n.id = ?1", Self::NOTE_SELECT),
                params![id],
                Self::fetch_note_row,
            )?;
        Self::enrich_note(&conn, &mut note)?;
        Ok(note)
    }

    pub fn update_note(
        &self,
        id: i64,
        title: &str,
        content: &str,
        category_id: Option<i64>,
    ) -> Result<Note, rusqlite::Error> {
        self.update_note_full(id, title, content, category_id, None, None)
    }

    pub fn update_note_full(
        &self,
        id: i64,
        title: &str,
        content: &str,
        category_id: Option<i64>,
        contact_id: Option<i64>,
        coworker_id: Option<i64>,
    ) -> Result<Note, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, category_id = ?3, contact_id = ?4, coworker_id = ?5, updated_at = ?6
             WHERE id = ?7",
            params![title, content, category_id, contact_id, coworker_id, now, id],
        )?;

        let mut note = conn.query_row(
            &format!("{} WHERE n.id = ?1", Self::NOTE_SELECT),
            params![id],
            Self::fetch_note_row,
        )?;
        Self::enrich_note(&conn, &mut note)?;
        Ok(note)
    }

    pub fn delete_note(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_note(&self, id: i64) -> Result<Note, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut note = conn.query_row(
            &format!("{} WHERE n.id = ?1", Self::NOTE_SELECT),
            params![id],
            Self::fetch_note_row,
        )?;
        Self::enrich_note(&conn, &mut note)?;
        Ok(note)
    }

    pub fn list_notes(&self, limit: i64, offset: i64) -> Result<Vec<Note>, rusqlite::Error> {
        self.list_notes_filtered(limit, offset, false)
    }

    pub fn list_notes_filtered(
        &self,
        limit: i64,
        offset: i64,
        include_archived: bool,
    ) -> Result<Vec<Note>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let sql = if include_archived {
            format!("{} ORDER BY n.pinned DESC, n.sort_order ASC, n.updated_at DESC LIMIT ?1 OFFSET ?2", Self::NOTE_SELECT)
        } else {
            format!("{} WHERE n.archived = 0 ORDER BY n.pinned DESC, n.sort_order ASC, n.updated_at DESC LIMIT ?1 OFFSET ?2", Self::NOTE_SELECT)
        };
        let mut stmt = conn.prepare(&sql)?;
        let mut notes = stmt
            .query_map(params![limit, offset], Self::fetch_note_row)?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        for note in &mut notes {
            Self::enrich_note(&conn, note)?;
        }
        Ok(notes)
    }

    pub fn list_notes_for_contact(&self, contact_id: i64) -> Result<Vec<Note>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!(
            "{} WHERE n.contact_id = ?1 AND n.archived = 0 ORDER BY n.updated_at DESC",
            Self::NOTE_SELECT
        ))?;
        let mut notes = stmt
            .query_map(params![contact_id], Self::fetch_note_row)?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        for note in &mut notes {
            Self::enrich_note(&conn, note)?;
        }
        Ok(notes)
    }

    pub fn archive_note(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE notes SET archived = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn unarchive_note(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE notes SET archived = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn pin_note(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE notes SET pinned = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn unpin_note(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("UPDATE notes SET pinned = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_note(&self, id: i64, new_sort_order: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notes SET sort_order = ?1 WHERE id = ?2",
            params![new_sort_order, id],
        )?;
        Ok(())
    }

    // --- Note links ---

    pub fn link_notes(&self, from_id: i64, to_id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO note_links (from_note_id, to_note_id) VALUES (?1, ?2)",
            params![from_id, to_id],
        )?;
        Ok(())
    }

    pub fn unlink_notes(&self, from_id: i64, to_id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM note_links WHERE from_note_id = ?1 AND to_note_id = ?2",
            params![from_id, to_id],
        )?;
        Ok(())
    }

    pub fn search_notes(&self, query: &str, limit: i64) -> Result<Vec<SearchResult>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        // Build a safe FTS5 query: wrap each word in double quotes for prefix matching
        // This prevents FTS5 syntax errors from special characters
        let fts_query: String = query
            .split_whitespace()
            .map(|w| format!("\"{}\"*", w.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" ");

        // Try FTS5 first
        if !fts_query.is_empty() {
            let mut stmt = conn.prepare(
                "SELECT n.id, n.title, n.content, snippet(notes_fts, 1, '<mark>', '</mark>', '…', 20) as snippet, n.updated_at
                 FROM notes_fts
                 JOIN notes n ON n.id = notes_fts.rowid
                 WHERE notes_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )?;
            let results = stmt
                .query_map(params![fts_query, limit], |row| {
                    Ok(SearchResult {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        content: row.get(2)?,
                        snippet: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            if !results.is_empty() {
                return Ok(results);
            }
        }

        // Fallback: LIKE-based substring search on title and content
        let like_pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = conn.prepare(
            "SELECT id, title, content, substr(content, 1, 200) as snippet, updated_at
             FROM notes
             WHERE title LIKE ?1 ESCAPE '\\' OR content LIKE ?1 ESCAPE '\\'
             ORDER BY updated_at DESC
             LIMIT ?2",
        )?;
        let results = stmt
            .query_map(params![like_pattern, limit], |row| {
                Ok(SearchResult {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    content: row.get(2)?,
                    snippet: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    // --- Tags ---

    pub fn list_tags(&self) -> Result<Vec<Tag>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM tags ORDER BY name")?;
        let tags = stmt
            .query_map([], |row| Ok(Tag { id: row.get(0)?, name: row.get(1)? }))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn get_tags_for_note(&self, note_id: i64) -> Result<Vec<Tag>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        Self::fetch_tags_for_note(&conn, note_id)
    }

    pub fn add_tag_to_note(&self, note_id: i64, tag_name: &str) -> Result<Tag, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
            params![tag_name],
        )?;
        let tag_id: i64 = conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            params![tag_name],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
            params![note_id, tag_id],
        )?;
        Ok(Tag {
            id: tag_id,
            name: tag_name.to_string(),
        })
    }

    pub fn remove_tag_from_note(&self, note_id: i64, tag_id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2",
            params![note_id, tag_id],
        )?;
        Ok(())
    }

    // --- Categories ---

    pub fn list_categories(&self) -> Result<Vec<Category>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, color FROM categories ORDER BY name")?;
        let cats = stmt
            .query_map([], |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(cats)
    }

    pub fn create_category(&self, name: &str, color: &str) -> Result<Category, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO categories (name, color) VALUES (?1, ?2)",
            params![name, color],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Category {
            id,
            name: name.to_string(),
            color: color.to_string(),
        })
    }

    // --- Reminders ---

    pub fn set_reminder(
        &self,
        note_id: i64,
        due_at: &str,
        recur_interval: Option<i64>,
        recur_unit: Option<&str>,
    ) -> Result<Reminder, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO reminders (note_id, due_at, fired, recur_interval, recur_unit)
             VALUES (?1, ?2, 0, ?3, ?4)
             ON CONFLICT(note_id) DO UPDATE SET due_at = excluded.due_at, fired = 0, calendar_event_id = NULL, recur_interval = excluded.recur_interval, recur_unit = excluded.recur_unit",
            params![note_id, due_at, recur_interval, recur_unit],
        )?;
        let reminder = conn.query_row(
            "SELECT id, note_id, due_at, fired, calendar_event_id, recur_interval, recur_unit FROM reminders WHERE note_id = ?1",
            params![note_id],
            |row| {
                Ok(Reminder {
                    id: row.get(0)?,
                    note_id: row.get(1)?,
                    due_at: row.get(2)?,
                    fired: row.get(3)?,
                    calendar_event_id: row.get(4)?,
                    recur_interval: row.get(5)?,
                    recur_unit: row.get(6)?,
                })
            },
        )?;
        Ok(reminder)
    }

    pub fn delete_reminder(&self, note_id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM reminders WHERE note_id = ?1", params![note_id])?;
        Ok(())
    }

    pub fn get_reminder_for_note(&self, note_id: i64) -> Result<Option<Reminder>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        Self::fetch_reminder_for_note(&conn, note_id)
    }

    pub fn update_reminder_calendar_event(
        &self,
        reminder_id: i64,
        calendar_event_id: &str,
    ) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE reminders SET calendar_event_id = ?1 WHERE id = ?2",
            params![calendar_event_id, reminder_id],
        )?;
        Ok(())
    }

    pub fn mark_reminder_fired(&self, reminder_id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE reminders SET fired = 1 WHERE id = ?1",
            params![reminder_id],
        )?;
        Ok(())
    }

    pub fn list_due_reminders(&self) -> Result<Vec<(Reminder, Note)>, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT r.id, r.note_id, r.due_at, r.fired, r.calendar_event_id, r.recur_interval, r.recur_unit,
                    n.id, n.title, n.content, n.category_id, n.created_at, n.updated_at, n.source,
                    n.contact_id, n.coworker_id, n.archived, n.sort_order, n.pinned
             FROM reminders r
             JOIN notes n ON n.id = r.note_id
             WHERE r.fired = 0 AND r.due_at <= ?1",
        )?;
        let rows = stmt
            .query_map(params![now], |row| {
                let reminder = Reminder {
                    id: row.get(0)?,
                    note_id: row.get(1)?,
                    due_at: row.get(2)?,
                    fired: row.get(3)?,
                    calendar_event_id: row.get(4)?,
                    recur_interval: row.get(5)?,
                    recur_unit: row.get(6)?,
                };
                let note = Note {
                    id: row.get(7)?,
                    title: row.get(8)?,
                    content: row.get(9)?,
                    category_id: row.get(10)?,
                    category_name: None,
                    category_color: None,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                    source: row.get(13)?,
                    tags: vec![],
                    reminder: None,
                    contact_id: row.get(14)?,
                    coworker_id: row.get(15)?,
                    contact: None,
                    coworker: None,
                    archived: row.get(16)?,
                    pinned: row.get(18)?,
                    sort_order: row.get(17)?,
                    links: vec![],
                    backlinks: vec![],
                };
                Ok((reminder, note))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // --- Contacts (customers) ---

    pub fn list_contacts(&self) -> Result<Vec<Contact>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, phone, mobile, created_at, updated_at
             FROM contacts ORDER BY last_name, first_name",
        )?;
        let contacts = stmt
            .query_map([], Self::row_to_contact)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(contacts)
    }

    pub fn search_contacts(&self, query: &str) -> Result<Vec<Contact>, rusqlite::Error> {
        let pattern = format!("%{}%", query);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, phone, mobile, created_at, updated_at
             FROM contacts
             WHERE last_name LIKE ?1 OR first_name LIKE ?1 OR email LIKE ?1 OR company_name LIKE ?1 OR customer_identifier LIKE ?1 OR phone LIKE ?1 OR mobile LIKE ?1
             ORDER BY last_name, first_name
             LIMIT 50",
        )?;
        let contacts = stmt
            .query_map(params![pattern], Self::row_to_contact)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(contacts)
    }

    pub fn create_contact(&self, c: &ContactInput) -> Result<Contact, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO contacts (last_name, first_name, address, email, gender, kind, company_name, customer_identifier, phone, mobile, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?11)",
            params![c.last_name, c.first_name, c.address, c.email, c.gender, c.kind, c.company_name, c.customer_identifier, c.phone, c.mobile, now],
        )?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, phone, mobile, created_at, updated_at
             FROM contacts WHERE id = ?1",
            params![id],
            Self::row_to_contact,
        )
    }

    pub fn update_contact(&self, id: i64, c: &ContactInput) -> Result<Contact, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE contacts SET last_name=?1, first_name=?2, address=?3, email=?4, gender=?5, kind=?6, company_name=?7, customer_identifier=?8, phone=?9, mobile=?10, updated_at=?11
             WHERE id=?12",
            params![c.last_name, c.first_name, c.address, c.email, c.gender, c.kind, c.company_name, c.customer_identifier, c.phone, c.mobile, now, id],
        )?;
        conn.query_row(
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, phone, mobile, created_at, updated_at
             FROM contacts WHERE id = ?1",
            params![id],
            Self::row_to_contact,
        )
    }

    pub fn delete_contact(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM contacts WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Coworkers ---

    pub fn list_coworkers(&self) -> Result<Vec<Coworker>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, last_name, first_name, email, created_at, updated_at
             FROM coworkers ORDER BY last_name, first_name",
        )?;
        let coworkers = stmt
            .query_map([], Self::row_to_coworker)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(coworkers)
    }

    pub fn search_coworkers(&self, query: &str) -> Result<Vec<Coworker>, rusqlite::Error> {
        let pattern = format!("%{}%", query);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, last_name, first_name, email, created_at, updated_at
             FROM coworkers
             WHERE last_name LIKE ?1 OR first_name LIKE ?1 OR email LIKE ?1
             ORDER BY last_name, first_name
             LIMIT 50",
        )?;
        let coworkers = stmt
            .query_map(params![pattern], Self::row_to_coworker)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(coworkers)
    }

    pub fn create_coworker(&self, c: &CoworkerInput) -> Result<Coworker, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO coworkers (last_name, first_name, email, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![c.last_name, c.first_name, c.email, now],
        )?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, last_name, first_name, email, created_at, updated_at
             FROM coworkers WHERE id = ?1",
            params![id],
            Self::row_to_coworker,
        )
    }

    pub fn update_coworker(&self, id: i64, c: &CoworkerInput) -> Result<Coworker, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE coworkers SET last_name=?1, first_name=?2, email=?3, updated_at=?4
             WHERE id=?5",
            params![c.last_name, c.first_name, c.email, now, id],
        )?;
        conn.query_row(
            "SELECT id, last_name, first_name, email, created_at, updated_at
             FROM coworkers WHERE id = ?1",
            params![id],
            Self::row_to_coworker,
        )
    }

    pub fn delete_coworker(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM coworkers WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Recent contacts/coworkers ---

    pub fn recent_contacts(&self, limit: i64) -> Result<Vec<Contact>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT c.id, c.last_name, c.first_name, c.address, c.email, c.gender, c.kind, c.company_name, c.customer_identifier, c.phone, c.mobile, c.created_at, c.updated_at
             FROM contacts c
             JOIN notes n ON n.contact_id = c.id
             WHERE n.archived = 0
             ORDER BY n.updated_at DESC
             LIMIT ?1",
        )?;
        let contacts = stmt
            .query_map(params![limit], Self::row_to_contact)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(contacts)
    }

    pub fn recent_coworkers(&self, limit: i64) -> Result<Vec<Coworker>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT c.id, c.last_name, c.first_name, c.email, c.created_at, c.updated_at
             FROM coworkers c
             JOIN notes n ON n.coworker_id = c.id
             WHERE n.archived = 0
             ORDER BY n.updated_at DESC
             LIMIT ?1",
        )?;
        let coworkers = stmt
            .query_map(params![limit], Self::row_to_coworker)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(coworkers)
    }

    // --- Note templates ---

    pub fn list_templates(&self) -> Result<Vec<NoteTemplate>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, title, content, category_id, tags FROM note_templates ORDER BY name",
        )?;
        let templates = stmt
            .query_map([], |row| {
                Ok(NoteTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category_id: row.get(4)?,
                    tags: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(templates)
    }

    pub fn create_template(&self, t: &TemplateInput) -> Result<NoteTemplate, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO note_templates (name, title, content, category_id, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![t.name, t.title, t.content, t.category_id, t.tags],
        )?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, name, title, content, category_id, tags FROM note_templates WHERE id = ?1",
            params![id],
            |row| {
                Ok(NoteTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    category_id: row.get(4)?,
                    tags: row.get(5)?,
                })
            },
        )
    }

    pub fn delete_template(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM note_templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Saved searches ---

    pub fn list_saved_searches(&self) -> Result<Vec<SavedSearch>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, query, category_id, tag_name, time_range FROM saved_searches ORDER BY name",
        )?;
        let searches = stmt
            .query_map([], |row| {
                Ok(SavedSearch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    query: row.get(2)?,
                    category_id: row.get(3)?,
                    tag_name: row.get(4)?,
                    time_range: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(searches)
    }

    pub fn create_saved_search(&self, s: &SavedSearchInput) -> Result<SavedSearch, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO saved_searches (name, query, category_id, tag_name, time_range) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![s.name, s.query, s.category_id, s.tag_name, s.time_range],
        )?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, name, query, category_id, tag_name, time_range FROM saved_searches WHERE id = ?1",
            params![id],
            |row| {
                Ok(SavedSearch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    query: row.get(2)?,
                    category_id: row.get(3)?,
                    tag_name: row.get(4)?,
                    time_range: row.get(5)?,
                })
            },
        )
    }

    pub fn delete_saved_search(&self, id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM saved_searches WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Statistics ---

    pub fn get_statistics(&self) -> Result<NoteStatistics, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();

        let total_notes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE archived = 0",
            [],
            |row| row.get(0),
        )?;

        let archived_notes: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE archived = 1",
            [],
            |row| row.get(0),
        )?;

        let notes_with_reminders: i64 = conn.query_row(
            "SELECT COUNT(*) FROM reminders WHERE fired = 0",
            [],
            |row| row.get(0),
        )?;

        let total_contacts: i64 = conn.query_row("SELECT COUNT(*) FROM contacts", [], |row| row.get(0))?;

        let total_coworkers: i64 = conn.query_row("SELECT COUNT(*) FROM coworkers", [], |row| row.get(0))?;

        let week_ago = chrono::Utc::now() - chrono::Duration::days(7);
        let week_iso = week_ago.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
        let notes_this_week: i64 = conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE archived = 0 AND created_at >= ?1",
            params![week_iso],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT c.name, c.color, COUNT(n.id) as cnt FROM categories c
             LEFT JOIN notes n ON n.category_id = c.id AND n.archived = 0
             GROUP BY c.id, c.name, c.color ORDER BY cnt DESC",
        )?;
        let notes_per_category = stmt
            .query_map([], |row| {
                Ok(CategoryCount {
                    name: row.get(0)?,
                    color: row.get(1)?,
                    count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(NoteStatistics {
            total_notes,
            archived_notes,
            notes_with_reminders,
            total_contacts,
            total_coworkers,
            notes_this_week,
            notes_per_category,
        })
    }

    // --- Recurring reminder re-arm ---

    pub fn rearm_recurring_reminder(&self, reminder_id: i64) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let (due_at, interval, unit): (String, Option<i64>, Option<String>) = conn.query_row(
            "SELECT due_at, recur_interval, recur_unit FROM reminders WHERE id = ?1",
            params![reminder_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        if let (Some(interval), Some(unit)) = (interval, unit) {
            let dt = chrono::DateTime::parse_from_rfc3339(&due_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?;
            let next = match unit.as_str() {
                "minutes" => dt + chrono::Duration::minutes(interval),
                "hours" => dt + chrono::Duration::hours(interval),
                "days" => dt + chrono::Duration::days(interval),
                "weeks" => dt + chrono::Duration::weeks(interval),
                "months" => dt + chrono::Duration::days(interval * 30),
                _ => return Ok(()), // unknown unit, don't rearm
            };
            let next_iso = next.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
            conn.execute(
                "UPDATE reminders SET due_at = ?1, fired = 0 WHERE id = ?2",
                params![next_iso, reminder_id],
            )?;
        }
        Ok(())
    }
}

// Input structs for create/update
pub struct ContactInput {
    pub last_name: String,
    pub first_name: String,
    pub address: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub kind: String,
    pub company_name: Option<String>,
    pub customer_identifier: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
}

pub struct CoworkerInput {
    pub last_name: String,
    pub first_name: String,
    pub email: Option<String>,
}

pub struct TemplateInput {
    pub name: String,
    pub title: String,
    pub content: String,
    pub category_id: Option<i64>,
    pub tags: String,
}

pub struct SavedSearchInput {
    pub name: String,
    pub query: String,
    pub category_id: Option<i64>,
    pub tag_name: Option<String>,
    pub time_range: Option<String>,
}

// Helper for tests
pub fn test_db_path() -> String {
    let dir = std::env::temp_dir();
    dir.join(format!("masternote_test_{}.db", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .to_string()
}
