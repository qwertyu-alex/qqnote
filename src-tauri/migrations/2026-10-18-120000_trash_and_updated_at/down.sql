-- Trashed notes come back as regular notes rather than being lost.
CREATE TABLE note_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    title varchar NOT NULL,
    body text NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

INSERT INTO note_old (id, title, body, created_at)
SELECT id, title, body, created_at FROM note;

DROP TABLE note;
ALTER TABLE note_old RENAME TO note;
