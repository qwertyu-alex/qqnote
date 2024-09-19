use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::{fs, path};

use dotenv::dotenv;
use std::env;

use crate::models::{NewNote, Note, NoteMeta};

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

pub fn create_note(
    conn: &mut SqliteConnection,
    title: &str,
    body: &str,
    optional_id: &Option<i32>,
) -> Note {
    use crate::schema::note;

    let new_note = NewNote {
        title,
        body,
        id: *optional_id,
    };

    diesel::insert_into(note::table)
        .values(&new_note)
        .on_conflict(note::id)
        .do_update()
        .set((note::title.eq(new_note.title), note::body.eq(new_note.body)))
        .returning(Note::as_returning())
        .get_result(conn)
        .expect("Error saving new note")
}

pub fn get_notes(conn: &mut SqliteConnection) -> Vec<NoteMeta> {
    use crate::schema::note;
    use crate::schema::note::id;

    note::dsl::note
        .limit(50)
        .select((note::id, note::title, note::created_at))
        .order(id.desc())
        .load::<NoteMeta>(conn)
        .expect("Error loading posts")
}

pub fn get_note_text(conn: &mut SqliteConnection, id: i32) -> String {
    use crate::schema::note;

    note::dsl::note
        .filter(note::id.eq(id))
        .select(note::body)
        .first(conn)
        .expect("Error loading posts")
}
