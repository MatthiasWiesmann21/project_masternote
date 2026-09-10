-- Migration v7: Note templates
CREATE TABLE IF NOT EXISTS note_templates (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    title       TEXT NOT NULL DEFAULT '',
    content     TEXT NOT NULL DEFAULT '',
    category_id INTEGER REFERENCES categories(id) ON DELETE SET NULL,
    tags        TEXT NOT NULL DEFAULT ''  -- comma-separated tag names
);
