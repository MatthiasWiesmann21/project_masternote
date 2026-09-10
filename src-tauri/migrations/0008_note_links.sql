-- Migration v8: Note links (backlinks)
CREATE TABLE IF NOT EXISTS note_links (
    from_note_id INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    to_note_id   INTEGER NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    PRIMARY KEY (from_note_id, to_note_id)
);
CREATE INDEX IF NOT EXISTS idx_note_links_to ON note_links(to_note_id);
