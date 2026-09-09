-- MasterNote schema v2: contacts + coworkers

CREATE TABLE IF NOT EXISTS contacts (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    last_name           TEXT NOT NULL DEFAULT '',
    first_name          TEXT NOT NULL DEFAULT '',
    address             TEXT,
    email               TEXT,
    gender              TEXT,
    kind                TEXT NOT NULL DEFAULT 'private',  -- 'private' or 'company'
    company_name        TEXT,
    customer_identifier TEXT,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_contacts_name ON contacts(last_name);
CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email);

CREATE TABLE IF NOT EXISTS coworkers (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    last_name   TEXT NOT NULL DEFAULT '',
    first_name  TEXT NOT NULL DEFAULT '',
    email       TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_coworkers_name ON coworkers(last_name);
CREATE INDEX IF NOT EXISTS idx_coworkers_email ON coworkers(email);

-- Add contact/coworker links to notes
ALTER TABLE notes ADD COLUMN contact_id INTEGER REFERENCES contacts(id) ON DELETE SET NULL;
ALTER TABLE notes ADD COLUMN coworker_id INTEGER REFERENCES coworkers(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_notes_contact ON notes(contact_id);
CREATE INDEX IF NOT EXISTS idx_notes_coworker ON notes(coworker_id);
