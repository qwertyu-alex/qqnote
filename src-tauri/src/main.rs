// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::models::{ExportedNote, ImportResult, NoteMeta};
use diesel::prelude::*;
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::{CheckMenuItemBuilder, Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Manager, RunEvent, Runtime, State, WindowEvent};

mod appearance;
mod db;
mod launcher;
mod models;
mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

// Define an AppState struct to hold the SQLite connection inside a Mutex
struct AppState {
    // None if the database couldn't be opened; see `startup_error`.
    db: Mutex<Option<SqliteConnection>>,
    startup_error: Option<String>,
}

/// A quit that is waiting for the frontend to save the open note.
#[derive(Default)]
struct QuitState {
    pending: AtomicBool,
    // Numbered so a timed-out fallback never acts on a newer request.
    requested: AtomicU64,
    acknowledged: AtomicU64,
}

fn with_db<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&mut SqliteConnection) -> QueryResult<T>,
) -> Result<T, String> {
    let mut db = state.db.lock().map_err(|e| e.to_string())?;
    let conn = db.as_mut().ok_or("The database isn't available")?;
    f(conn).map_err(|e| e.to_string())
}

/// Why the database couldn't be opened, if it couldn't. The frontend disables editing then.
#[tauri::command]
fn startup_error(state: State<'_, AppState>) -> Option<String> {
    state.startup_error.clone()
}

// Commands run off the main thread (`async`) so large saves never stall the window.
// The frontend sends writes one at a time, so they can't reorder.

#[tauri::command(async)]
fn save_note(id: Option<i32>, body: String, state: State<'_, AppState>) -> Result<i32, String> {
    with_db(&state, |conn| db::save_note(conn, id, &body))
}

#[tauri::command(async)]
fn discard_note(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    with_db(&state, |conn| db::discard_note(conn, id))
}

#[tauri::command(async)]
fn get_notes(state: State<'_, AppState>) -> Result<Vec<NoteMeta>, String> {
    with_db(&state, db::get_notes)
}

#[tauri::command(async)]
fn get_trash(state: State<'_, AppState>) -> Result<Vec<NoteMeta>, String> {
    with_db(&state, db::get_trash)
}

#[tauri::command(async)]
fn get_note_text(id: i32, state: State<'_, AppState>) -> Result<String, String> {
    with_db(&state, |conn| db::get_note_text(conn, id))
}

#[tauri::command(async)]
fn trash_note(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    with_db(&state, |conn| db::trash_note(conn, id))
}

#[tauri::command(async)]
fn restore_note(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    with_db(&state, |conn| db::restore_note(conn, id))
}

#[tauri::command(async)]
fn delete_forever(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    with_db(&state, |conn| db::delete_forever(conn, id))
}

#[tauri::command(async)]
fn empty_trash(state: State<'_, AppState>) -> Result<usize, String> {
    with_db(&state, db::empty_trash)
}

#[tauri::command(async)]
fn export_notes(path: String, state: State<'_, AppState>) -> Result<usize, String> {
    let notes = with_db(&state, db::export_notes)?;
    let json = serde_json::to_string_pretty(&notes).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write {}: {}", path, e))?;
    Ok(notes.len())
}

#[tauri::command(async)]
fn import_notes(path: String, state: State<'_, AppState>) -> Result<ImportResult, String> {
    let json =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
    let notes: Vec<ExportedNote> =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    with_db(&state, |conn| db::import_notes(conn, notes))
}

/// Asks the frontend to save the open note, then quit. Quits anyway if the frontend
/// doesn't acknowledge within two seconds (e.g. the page is hung).
fn request_quit<R: Runtime>(app: &AppHandle<R>) {
    let quit = app.state::<QuitState>();
    if quit.pending.swap(true, Ordering::SeqCst) {
        return;
    }
    let request = quit.requested.fetch_add(1, Ordering::SeqCst) + 1;

    if app.emit("flush-and-quit", ()).is_err() {
        app.exit(0);
        return;
    }

    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        if app.state::<QuitState>().acknowledged.load(Ordering::SeqCst) < request {
            app.exit(0);
        }
    });
}

#[tauri::command]
fn quit_acknowledged(quit: State<'_, QuitState>) {
    let latest = quit.requested.load(Ordering::SeqCst);
    quit.acknowledged.store(latest, Ordering::SeqCst);
}

#[tauri::command]
fn quit_cancelled(quit: State<'_, QuitState>) {
    quit.pending.store(false, Ordering::SeqCst);
}

#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

fn main() {
    println!("App started!");

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .arg(launcher::BACKGROUND_ARG)
                .build(),
        )
        .manage(QuitState::default())
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
                "open_at_login" => launcher::toggle_open_at_login(app),
                "quit" => request_quit(app),
                id => {
                    if let Some(appearance) = appearance::Appearance::from_menu_id(id) {
                        appearance::select(app, appearance);
                    }
                }
            }
        })
        .on_window_event(|window, event| {
            // Closing only hides the window, so Caps Lock can bring it back. ⌘Q quits.
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
            // The Mac switched between light and dark (only matters while following it).
            if let WindowEvent::ThemeChanged(_) = event {
                appearance::update_dock_icon(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            save_note,
            discard_note,
            get_notes,
            get_trash,
            get_note_text,
            trash_note,
            restore_note,
            delete_forever,
            empty_trash,
            export_notes,
            import_notes,
            quit_acknowledged,
            quit_cancelled,
            quit,
            startup_error
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| match event {
        // `code` is None when the user (not `app.exit`) triggered the exit.
        RunEvent::ExitRequested {
            code: None, api, ..
        } => {
            api.prevent_exit();
            request_quit(app);
        }
        // Clicking the Dock icon while the window is closed.
        #[cfg(target_os = "macos")]
        RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } => launcher::show(app),
        // After Tauri sets its own Dock icon in dev builds.
        RunEvent::Ready => appearance::update_dock_icon(app),
        RunEvent::Exit => launcher::teardown(),
        _ => {}
    });
}

fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    // Create menu items
    let new_note = MenuItemBuilder::with_id("new_note", "New Note")
        .accelerator("CommandOrControl+T")
        .build(app)?;
    let export = MenuItemBuilder::with_id("export", "Export Notes").build(app)?;
    let import = MenuItemBuilder::with_id("import", "Import Notes").build(app)?;
    // Checked in `launcher::setup`, once the autostart plugin can say if it's on.
    let open_at_login =
        CheckMenuItemBuilder::with_id("open_at_login", "Open at Login").build(app)?;
    let file_submenu = SubmenuBuilder::with_id(app, "file", "File")
        .item(&new_note)
        .item(&export)
        .item(&import)
        .separator()
        .item(&open_at_login)
        .build()?;

    // The native Quit item terminates the process immediately; this one saves first.
    let quit = MenuItemBuilder::with_id("quit", "Quit qqnote")
        .accelerator("CommandOrControl+Q")
        .build(app)?;

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
        .item(&quit)
        .close_window()
        .build()?;

    MenuBuilder::new(app)
        .item(&file_submenu)
        .item(&edit_submenu)
        .item(&appearance::view_submenu(app)?)
        .build()
}

fn setup_handler(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let main_window = app.get_webview_window("main").unwrap();

    // If this fails, the window explains why instead of the app vanishing.
    let (connection, startup_error) = match db::establish_connection() {
        Ok(connection) => (Some(connection), None),
        Err(e) => {
            eprintln!("Couldn't open the database: {}", e);
            (None, Some(e.to_string()))
        }
    };

    if cfg!(debug_assertions) {
        // Create a string formatter that takes in the current title and appends "in developing" to it
        let original_title = main_window.title().unwrap();
        let title = format!("{} in developing", original_title);
        main_window.set_title(&title).unwrap();
    }

    app.manage(AppState {
        db: Mutex::new(connection), // Store the connection in the AppState, wrapped in a Mutex
        startup_error,
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

    appearance::setup(app.handle());
    launcher::setup(app.handle());

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
