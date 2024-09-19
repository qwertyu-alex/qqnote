use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::{fs, path};

use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let _env = env::var("APP_ENV");

    match _env {
        Ok(_env) => {
            let database_url = &env::var("DATABASE_URL").unwrap();

            SqliteConnection::establish(&database_url)
                .expect(&format!("Error connecting to {}", &database_url))
        }
        Err(_) => {
            println!("no APP_ENV");
            let project_dir = &tauri::api::path::home_dir().unwrap().join(".qqnote");
            fs::create_dir_all(project_dir).unwrap();

            let database_url = project_dir.join("app_data.db");
            let database_url = database_url.to_str().unwrap();

            SqliteConnection::establish(&database_url)
                .expect(&format!("Error connecting to {}", &database_url))
        }
    }
}

use crate::models::{NewNote, Note};
pub fn create_note(conn: &mut SqliteConnection, title: &str, body: &str) -> Note {
    use crate::schema::note;

    let new_note = NewNote { title, body };

    diesel::insert_into(note::table)
        .values(&new_note)
        .returning(Note::as_returning())
        .get_result(conn)
        .expect("Error saving new note")
}

pub fn get_notes(conn: &mut SqliteConnection) -> Vec<Note> {
    use crate::schema::note::dsl::note;

    note.limit(5)
        .select(Note::as_select())
        .load(conn)
        .expect("Error loading posts")
}
