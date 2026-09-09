use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

use crate::models::{Category, Contact, Coworker, Note, Reminder, SearchResult, Tag};

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
        let sql1 = include_str!("../migrations/0001_init.sql");
        let sql2 = include_str!("../migrations/0002_contacts_coworkers.sql");
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(sql1)?;
        // v2 uses ALTER TABLE which can fail if column already exists — ignore that case
        for stmt in sql2.split(';') {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Err(e) = conn.execute(trimmed, []) {
                // "duplicate column name" is expected on re-run
                let msg = e.to_string();
                if !msg.contains("duplicate column name") {
                    return Err(Box::new(e));
                }
            }
        }
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

    fn fetch_contact(conn: &Connection, contact_id: Option<i64>) -> Result<Option<Contact>, rusqlite::Error> {
        if let Some(cid) = contact_id {
            conn.query_row(
                "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, created_at, updated_at
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
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
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
        })
    }

    const NOTE_SELECT: &'static str =
        "SELECT n.id, n.title, n.content, n.category_id, c.name, c.color, \
         n.created_at, n.updated_at, n.source, n.contact_id, n.coworker_id \
         FROM notes n \
         LEFT JOIN categories c ON n.category_id = c.id";

    fn enrich_note(conn: &Connection, note: &mut Note) -> Result<(), rusqlite::Error> {
        note.tags = Self::fetch_tags_for_note(conn, note.id)?;
        note.reminder = Self::fetch_reminder_for_note(conn, note.id)?;
        note.contact = Self::fetch_contact(conn, note.contact_id)?;
        note.coworker = Self::fetch_coworker(conn, note.coworker_id)?;
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
            Self::enrich_note(&conn, note)?;
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
                    n.id, n.title, n.content, n.category_id, n.created_at, n.updated_at, n.source,
                    n.contact_id, n.coworker_id
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
                    contact_id: row.get(12)?,
                    coworker_id: row.get(13)?,
                    contact: None,
                    coworker: None,
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
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, created_at, updated_at
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
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, created_at, updated_at
             FROM contacts
             WHERE last_name LIKE ?1 OR first_name LIKE ?1 OR email LIKE ?1 OR company_name LIKE ?1 OR customer_identifier LIKE ?1
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
            "INSERT INTO contacts (last_name, first_name, address, email, gender, kind, company_name, customer_identifier, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
            params![c.last_name, c.first_name, c.address, c.email, c.gender, c.kind, c.company_name, c.customer_identifier, now],
        )?;
        let id = conn.last_insert_rowid();
        conn.query_row(
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, created_at, updated_at
             FROM contacts WHERE id = ?1",
            params![id],
            Self::row_to_contact,
        )
    }

    pub fn update_contact(&self, id: i64, c: &ContactInput) -> Result<Contact, rusqlite::Error> {
        let now = Self::now_iso();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE contacts SET last_name=?1, first_name=?2, address=?3, email=?4, gender=?5, kind=?6, company_name=?7, customer_identifier=?8, updated_at=?9
             WHERE id=?10",
            params![c.last_name, c.first_name, c.address, c.email, c.gender, c.kind, c.company_name, c.customer_identifier, now, id],
        )?;
        conn.query_row(
            "SELECT id, last_name, first_name, address, email, gender, kind, company_name, customer_identifier, created_at, updated_at
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
}

pub struct CoworkerInput {
    pub last_name: String,
    pub first_name: String,
    pub email: Option<String>,
}

// Helper for tests
pub fn test_db_path() -> String {
    let dir = std::env::temp_dir();
    dir.join(format!("masternote_test_{}.db", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .to_string()
}
