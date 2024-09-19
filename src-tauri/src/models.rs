use crate::schema::note;
use diesel::prelude::*;
use diesel::sql_types::{Text, Timestamp};
use serde::Serialize;

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::note)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = note)]
pub struct NewNote<'a> {
    pub title: &'a str,
    pub body: &'a str,
}
