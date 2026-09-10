-- Migration v5: Note archiving
ALTER TABLE notes ADD COLUMN archived INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_notes_archived ON notes(archived);
