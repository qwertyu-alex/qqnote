// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::models::NoteMeta;
use diesel::prelude::*;
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

#[tauri::command]
fn delete_note(id: i32, state: State<'_, AppState>) -> bool {
    let mut conn = state.db.lock().unwrap();

    let result = db::delete_note(&mut conn, id);
    result
}

#[tauri::command]
fn export_notes(state: State<'_, AppState>) -> Vec<Note> {
    println!("Exporting notes from database...");
    let mut conn = state.db.lock().unwrap();
    use crate::schema::note;
    let notes = note::dsl::note
        .select((note::id, note::title, note::body, note::created_at))
        .load(&mut *conn)
        .expect("Error loading notes for export");
    println!("Successfully loaded {} notes for export", notes.len());
    notes
}

fn main() {
    let mut connection = db::establish_connection();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Error migrating");

    println!("App started!");

    // Create menu items
    let export = CustomMenuItem::new("export".to_string(), "Export Notes");
    let file_submenu = Submenu::new("File", Menu::new().add_item(export));

    // Create Edit submenu with native items
    let edit_submenu = Submenu::new(
        "Edit",
        Menu::new()
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::SelectAll),
    );

    let menu = Menu::new()
        .add_submenu(file_submenu)
        .add_submenu(edit_submenu);

    tauri::Builder::default()
        .setup(setup_handler)
        .menu(menu)
        .on_menu_event(|event| {
            println!("Menu event received: {}", event.menu_item_id());
            match event.menu_item_id() {
                "export" => {
                    println!("Export menu item clicked, emitting event...");
                    event.window().emit("export", ()).unwrap();
                    println!("Export event emitted");
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_notes,
            create_note,
            get_note_text,
            delete_note,
            export_notes
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
