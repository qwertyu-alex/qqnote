use crate::schema::note;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = note)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub body: String,
}

/// What the history and trash lists need; never includes the body.
#[derive(Selectable, Serialize, Queryable)]
#[diesel(table_name = note)]
pub struct NoteMeta {
    pub id: i32,
    pub title: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

/// A note in an export file. Files exported by older versions only have
/// `id`, `title`, `body` and `created_at`, so everything else is optional.
#[derive(Serialize, Deserialize)]
pub struct ExportedNote {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub title: String,
    pub body: String,
    pub created_at: chrono::NaiveDateTime,
    #[serde(default)]
    pub updated_at: Option<chrono::NaiveDateTime>,
    #[serde(default)]
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl From<Note> for ExportedNote {
    fn from(n: Note) -> Self {
        ExportedNote {
            id: n.id,
            title: n.title,
            body: n.body,
            created_at: n.created_at,
            updated_at: Some(n.updated_at),
            deleted_at: n.deleted_at,
        }
    }
}

#[derive(Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}
