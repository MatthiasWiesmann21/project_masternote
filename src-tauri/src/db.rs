use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::models::{Category, Note, Reminder, SearchResult, Tag};

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
        let sql = include_str!("../migrations/0001_init.sql");
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql)?;
        Ok(())
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
                "SELECT id, note_id, due_at, fired, calendar_event_id FROM reminders WHERE note_id = ?1",
                params![note_id],
                |row| {
                    Ok(Reminder {
                        id: row.get(0)?,
                        note_id: row.get(1)?,
                        due_at: row.get(2)?,
                        fired: row.get(3)?,
                        calendar_event_id: row.get(4)?,
                    })
                },
            )
            .optional()?;
        Ok(reminder)
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
        })
    }

    const NOTE_SELECT: &'static str =
        "SELECT n.id, n.title, n.content, n.category_id, c.name, c.color, \
         n.created_at, n.updated_at, n.source \
         FROM notes n \
         LEFT JOIN categories c ON n.category_id = c.id";

    // --- Public API ---

    pub fn create_note(
        &self,
        title: &str,
        content: &str,
        category_id: Option<i64>,
    ) -> Result<Note, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO notes (title, content, category_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![title, content, category_id, now],
        )?;
        let id = conn.last_insert_rowid();

        let mut note = conn
            .query_row(
                &format!("{} WHERE n.id = ?1", Self::NOTE_SELECT),
                params![id],
                Self::fetch_note_row,
            )?;
        note.tags = Self::fetch_tags_for_note(&conn, id)?;
        note.reminder = Self::fetch_reminder_for_note(&conn, id)?;
        Ok(note)
    }

    pub fn update_note(
        &self,
        id: i64,
        title: &str,
        content: &str,
        category_id: Option<i64>,
    ) -> Result<Note, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE notes SET title = ?1, content = ?2, category_id = ?3, updated_at = ?4
             WHERE id = ?5",
            params![title, content, category_id, now, id],
        )?;

        let mut note = conn.query_row(
            &format!("{} WHERE n.id = ?1", Self::NOTE_SELECT),
            params![id],
            Self::fetch_note_row,
        )?;
        note.tags = Self::fetch_tags_for_note(&conn, id)?;
        note.reminder = Self::fetch_reminder_for_note(&conn, id)?;
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
        note.tags = Self::fetch_tags_for_note(&conn, id)?;
        note.reminder = Self::fetch_reminder_for_note(&conn, id)?;
        Ok(note)
    }

    pub fn list_notes(&self, limit: i64, offset: i64) -> Result<Vec<Note>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!(
            "{} ORDER BY n.updated_at DESC LIMIT ?1 OFFSET ?2",
            Self::NOTE_SELECT
        ))?;
        let mut notes = stmt
            .query_map(params![limit, offset], Self::fetch_note_row)?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);

        for note in &mut notes {
            note.tags = Self::fetch_tags_for_note(&conn, note.id)?;
            note.reminder = Self::fetch_reminder_for_note(&conn, note.id)?;
        }
        Ok(notes)
    }

    pub fn search_notes(&self, query: &str, limit: i64) -> Result<Vec<SearchResult>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.title, n.content, snippet(notes_fts, 1, '<mark>', '</mark>', '…', 20) as snippet, n.updated_at
             FROM notes_fts
             JOIN notes n ON n.id = notes_fts.rowid
             WHERE notes_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let results = stmt
            .query_map(params![query, limit], |row| {
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

    pub fn set_reminder(
        &self,
        note_id: i64,
        due_at: &str,
    ) -> Result<Reminder, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO reminders (note_id, due_at, fired)
             VALUES (?1, ?2, 0)
             ON CONFLICT(note_id) DO UPDATE SET due_at = excluded.due_at, fired = 0, calendar_event_id = NULL",
            params![note_id, due_at],
        )?;
        let reminder = conn.query_row(
            "SELECT id, note_id, due_at, fired, calendar_event_id FROM reminders WHERE note_id = ?1",
            params![note_id],
            |row| {
                Ok(Reminder {
                    id: row.get(0)?,
                    note_id: row.get(1)?,
                    due_at: row.get(2)?,
                    fired: row.get(3)?,
                    calendar_event_id: row.get(4)?,
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
            "SELECT r.id, r.note_id, r.due_at, r.fired, r.calendar_event_id,
                    n.id, n.title, n.content, n.category_id, n.created_at, n.updated_at, n.source
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
                };
                let note = Note {
                    id: row.get(5)?,
                    title: row.get(6)?,
                    content: row.get(7)?,
                    category_id: row.get(8)?,
                    category_name: None,
                    category_color: None,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    source: row.get(11)?,
                    tags: vec![],
                    reminder: None,
                };
                Ok((reminder, note))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }
}

// Helper for tests
pub fn test_db_path() -> String {
    let dir = std::env::temp_dir();
    dir.join(format!("masternote_test_{}.db", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .to_string()
}
