-- Adds updated_at (history sorts by last edit) and deleted_at (trash), and moves body to the
-- last column so listing notes never has to read through large note bodies.
CREATE TABLE note_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    title TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at DATETIME,
    body TEXT NOT NULL
);

INSERT INTO note_new (id, title, created_at, updated_at, deleted_at, body)
SELECT id, title, created_at, created_at, NULL, body FROM note;

-- Keep the AUTOINCREMENT counter so ids of deleted notes are never reused.
UPDATE sqlite_sequence
SET seq = (SELECT seq FROM sqlite_sequence WHERE name = 'note')
WHERE name = 'note_new' AND EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'note');

DROP TABLE note;
ALTER TABLE note_new RENAME TO note;

CREATE INDEX note_deleted_at_updated_at ON note (deleted_at, updated_at);
