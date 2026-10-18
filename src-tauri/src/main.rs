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
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};

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

#[tauri::command]
fn import_notes(json_path: String, state: State<'_, AppState>) -> Result<Vec<NoteMeta>, String> {
    println!("Importing notes from: {}", json_path);
    let mut conn = state.db.lock().unwrap();

    // Read and parse the JSON file
    let json_content =
        std::fs::read_to_string(&json_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let notes: Vec<Note> =
        serde_json::from_str(&json_content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut imported_notes = Vec::new();

    for note in notes {
        // Create new note without ID to avoid conflicts
        let new_id = db::create_note(&mut conn, &note.title, &note.body, &None);
        if let Ok(meta) = db::get_note_meta(&mut conn, new_id) {
            imported_notes.push(meta);
        }
    }

    Ok(imported_notes)
}

fn main() {
    let mut connection = db::establish_connection();
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Error migrating");

    println!("App started!");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(setup_handler)
        .menu(build_menu)
        .on_menu_event(|app, event| {
            println!("Menu event received: {}", event.id().as_ref());
            match event.id().as_ref() {
                "export" => {
                    println!("Export menu item clicked, emitting event...");
                    app.emit("export", ()).unwrap();
                    println!("Export event emitted");
                }
                "import" => {
                    println!("Import menu item clicked, emitting event...");
                    app.emit("import", ()).unwrap();
                    println!("Import event emitted");
                }
                "new_note" => {
                    println!("New note menu item clicked, emitting event...");
                    app.emit("new_note", ()).unwrap();
                    println!("New note event emitted");
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_notes,
            create_note,
            get_note_text,
            delete_note,
            export_notes,
            import_notes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    // Create menu items
    let new_note = MenuItemBuilder::with_id("new_note", "New Note")
        .accelerator("CommandOrControl+T")
        .build(app)?;
    let export = MenuItemBuilder::with_id("export", "Export Notes").build(app)?;
    let import = MenuItemBuilder::with_id("import", "Import Notes").build(app)?;
    let file_submenu = SubmenuBuilder::new(app, "File")
        .item(&new_note)
        .item(&export)
        .item(&import)
        .build()?;

    // Create Edit submenu with native items
    let edit_submenu = SubmenuBuilder::new(app, "Edit")
        .copy()
        .paste()
        .select_all()
        .show_all()
        .fullscreen()
        .minimize()
        .undo()
        .redo()
        .maximize()
        .cut()
        .quit()
        .close_window()
        .build()?;

    MenuBuilder::new(app)
        .item(&file_submenu)
        .item(&edit_submenu)
        .build()
}

fn setup_handler(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let connection = db::establish_connection();

    let main_window = app.get_webview_window("main").unwrap();

    match env::var("APP_ENV") {
        Ok(_env) => {
            // Create a string formatter that takes in the current title and appends "in developing" to it
            let original_title = main_window.title().unwrap();
            let title = format!("{} in developing", original_title);
            main_window.set_title(&title).unwrap();
        }
        Err(_) => {}
    }

    app.manage(AppState {
        db: Mutex::new(connection), // Store the connection in the AppState, wrapped in a Mutex
    });

    let path = app.path();
    for dir in [
        path.resource_dir(),
        path.app_config_dir(),
        path.app_data_dir(),
        path.app_local_data_dir(),
        path.app_cache_dir(),
        path.app_log_dir(),
        path.data_dir(),
        path.local_data_dir(),
        path.cache_dir(),
        path.config_dir(),
        path.executable_dir(),
        path.public_dir(),
        path.runtime_dir(),
        path.template_dir(),
        path.font_dir(),
        path.home_dir(),
        path.audio_dir(),
        path.desktop_dir(),
        path.document_dir(),
        path.download_dir(),
        path.picture_dir(),
    ] {
        println!("{}", dir.unwrap_or_default().to_string_lossy());
    }

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
