-- Migration v6: Note sort order (for drag-and-drop reordering)
ALTER TABLE notes ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_notes_sort_order ON notes(sort_order);
