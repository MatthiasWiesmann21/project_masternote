-- Migration v9: Saved searches / smart lists
CREATE TABLE IF NOT EXISTS saved_searches (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    query       TEXT NOT NULL DEFAULT '',
    category_id INTEGER,
    tag_name    TEXT,
    time_range  TEXT
);
