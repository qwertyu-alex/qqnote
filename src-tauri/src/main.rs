// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::models::NoteMeta;
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use models::Note;
use serde::Serialize;
use std::env;
use std::path::Path;
use std::sync::Mutex;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::{Manager, State};

mod db;
mod models;
mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

// Define an AppState struct to hold the SQLite connection inside a Mutex
struct AppState {
    db: Mutex<SqliteConnection>,
}

#[tauri::command]
fn create_note(title: String, body: String, id: Option<i32>, state: State<'_, AppState>) -> i32 {
    let mut conn = state.db.lock().unwrap();

    let note_id = db::create_note(&mut conn, &title, &body, &id);
    note_id
}

#[tauri::command]
fn get_notes(state: State<'_, AppState>) -> Vec<NoteMeta> {
    let mut conn = state.db.lock().unwrap();

    let result = db::get_notes(&mut conn);
    result
}

#[tauri::command]
fn get_note_text(id: i32, state: State<'_, AppState>) -> String {
    let mut conn = state.db.lock().unwrap();

    let result = db::get_note_text(&mut conn, id);
    result
}

fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    // let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    // let close = CustomMenuItem::new("close".to_string(), "Close");
    // let submenu = Submenu::new("File", Menu::new().add_item(quit).add_item(close));
    // let menu = Menu::new()
    //     .add_native_item(MenuItem::Copy)
    //     .add_item(CustomMenuItem::new("hide", "Hide"))
    //     .add_submenu(submenu);
    let mut connection = db::establish_connection();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Error migrating");

    println!("App started!");

    tauri::Builder::default()
        .setup(setup_handler)
        // .menu(menu)
        .invoke_handler(tauri::generate_handler![
            get_notes,
            create_note,
            get_note_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_handler(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut connection = db::establish_connection();

    let app_handle = app.handle();
    let main_window = app.get_window("main").unwrap();

    match env::var("APP_ENV") {
        Ok(_env) => {
            // Create a string formatter that takes in the current title and appends "in developing" to it
            let original_title = main_window.get_window("main").unwrap().title().unwrap();
            let title = format!("{} in developing", original_title);
            main_window.set_title(&title).unwrap();
        }
        Err(_) => {}
    }

    // let app_data_dir_path = app_handle
    //     .path_resolver()
    //     .app_data_dir()
    //     .unwrap_or(std::path::PathBuf::new());

    // let db_name = "app_data.db";
    // let dir = Path::new(&app_data_dir_path);
    // let production_path = dir.join(db_name).to_str().unwrap().to_string();

    // Initialize the SQLite database at app startup
    // let conn = Connection::open(path).expect("Failed to open the database");

    // initialize_db(&conn).unwrap();

    app.manage(AppState {
        db: Mutex::new(connection), // Store the connection in the AppState, wrapped in a Mutex
    });

    println!(
        "{}",
        app_handle
            .path_resolver()
            .resource_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        app_handle
            .path_resolver()
            .app_config_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        app_handle
            .path_resolver()
            .app_data_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        app_handle
            .path_resolver()
            .app_local_data_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        app_handle
            .path_resolver()
            .app_cache_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        app_handle
            .path_resolver()
            .app_log_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::data_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::local_data_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::cache_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::config_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::executable_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::public_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::runtime_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::template_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::font_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::home_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::audio_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::desktop_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::document_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::download_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );
    println!(
        "{}",
        tauri::api::path::picture_dir()
            .unwrap_or(std::path::PathBuf::new())
            .to_string_lossy()
    );

    Ok(())
}

// fn db_test(id: &String, text: &String, conn: &Connection) -> Result<()> {
//     conn.execute(
//         "
//         INSERT INTO note (id, text) VALUES (?1, ?2)
//             ON CONFLICT(id) DO UPDATE SET text = ?2
//         ",
//         [id, text],
//     )?;

//     let mut stmt = conn.prepare("SELECT id, text, created_at FROM note")?;

//     let person_iter = stmt.query_map([], |row| {
//         Ok(Note {
//             id: row.get(0)?,
//             text: row.get(1)?,
//             created_at: row.get(2)?,
//         })
//     })?;

//     for person in person_iter {
//         println!("Found person {:?}", person.unwrap());
//     }

//     Ok(())
// }
