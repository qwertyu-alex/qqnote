use diesel::connection::SimpleConnection;
use diesel::dsl::now;
use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::MigrationHarness;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{ExportedNote, ImportResult, Note, NoteMeta};
use crate::schema::note;
use crate::MIGRATIONS;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Release builds always use ~/.qqnote/app_data.db. Debug builds use DATABASE_URL (see
/// src-tauri/.env) or a separate dev database, so development never touches real notes.
pub fn database_path() -> Result<PathBuf, Error> {
    let dir = dirs::home_dir()
        .ok_or("couldn't find the home directory")?
        .join(".qqnote");

    if cfg!(debug_assertions) {
        dotenvy::dotenv().ok();
        return Ok(std::env::var("DATABASE_URL")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dir.join("dev_app_data.db")));
    }

    Ok(dir.join("app_data.db"))
}

/// Opens the database. Pending migrations run only after a backup has been written.
pub fn establish_connection() -> Result<SqliteConnection, Error> {
    let path = database_path()?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let url = path.to_str().ok_or("database path isn't valid UTF-8")?;
    let mut conn = SqliteConnection::establish(url)?;
    // Wait for other connections instead of failing immediately with SQLITE_BUSY.
    conn.batch_execute("PRAGMA busy_timeout = 5000;")?;

    if conn.has_pending_migration(MIGRATIONS)? {
        if !conn.applied_migrations()?.is_empty() {
            backup(&mut conn, &path)?;
        }
        conn.run_pending_migrations(MIGRATIONS)?;
        // Migrations that rebuild tables leave the old pages behind.
        conn.batch_execute("VACUUM;")?;
    }

    Ok(conn)
}

/// Writes a consistent copy of the database to `backups/` next to it.
fn backup(conn: &mut SqliteConnection, db_path: &Path) -> Result<PathBuf, Error> {
    let dir = db_path.parent().unwrap_or(Path::new("")).join("backups");
    fs::create_dir_all(&dir)?;

    let file = dir.join(format!(
        "app_data-{}.db",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    ));
    diesel::sql_query("VACUUM INTO ?")
        .bind::<Text, _>(file.to_str().ok_or("backup path isn't valid UTF-8")?)
        .execute(conn)?;

    Ok(file)
}

/// First 70 characters, counted in chars so multi-byte characters are never split.
fn title_of(body: &str) -> String {
    body.chars().take(70).collect()
}

pub fn save_note(conn: &mut SqliteConnection, id: Option<i32>, body: &str) -> QueryResult<i32> {
    let title = title_of(body);

    if let Some(id) = id {
        let updated = diesel::update(note::table.find(id))
            .set((
                note::title.eq(&title),
                note::body.eq(body),
                note::updated_at.eq(now),
            ))
            .execute(conn)?;
        if updated > 0 {
            return Ok(id);
        }
        // The note no longer exists; keep the text as a new note instead of dropping it.
    }

    diesel::insert_into(note::table)
        .values((note::title.eq(&title), note::body.eq(body)))
        .returning(note::id)
        .get_result(conn)
}

/// Removes a new note that was emptied before being kept.
pub fn discard_note(conn: &mut SqliteConnection, id: i32) -> QueryResult<()> {
    diesel::delete(note::table.find(id).filter(note::deleted_at.is_null())).execute(conn)?;
    Ok(())
}

pub fn get_notes(conn: &mut SqliteConnection) -> QueryResult<Vec<NoteMeta>> {
    note::table
        .filter(note::deleted_at.is_null())
        .order((note::updated_at.desc(), note::id.desc()))
        .select(NoteMeta::as_select())
        .load(conn)
}

pub fn get_trash(conn: &mut SqliteConnection) -> QueryResult<Vec<NoteMeta>> {
    note::table
        .filter(note::deleted_at.is_not_null())
        .order((note::deleted_at.desc(), note::id.desc()))
        .select(NoteMeta::as_select())
        .load(conn)
}

pub fn get_note_text(conn: &mut SqliteConnection, id: i32) -> QueryResult<String> {
    note::table.find(id).select(note::body).first(conn)
}

pub fn trash_note(conn: &mut SqliteConnection, id: i32) -> QueryResult<()> {
    diesel::update(note::table.find(id))
        .set(note::deleted_at.eq(now.nullable()))
        .execute(conn)?;
    Ok(())
}

pub fn restore_note(conn: &mut SqliteConnection, id: i32) -> QueryResult<()> {
    diesel::update(note::table.find(id))
        .set(note::deleted_at.eq(None::<chrono::NaiveDateTime>))
        .execute(conn)?;
    Ok(())
}

/// Only notes that are already in the trash can be deleted permanently.
pub fn delete_forever(conn: &mut SqliteConnection, id: i32) -> QueryResult<()> {
    diesel::delete(note::table.find(id).filter(note::deleted_at.is_not_null())).execute(conn)?;
    Ok(())
}

pub fn empty_trash(conn: &mut SqliteConnection) -> QueryResult<usize> {
    diesel::delete(note::table.filter(note::deleted_at.is_not_null())).execute(conn)
}

pub fn export_notes(conn: &mut SqliteConnection) -> QueryResult<Vec<ExportedNote>> {
    let notes = note::table
        .order(note::id.asc())
        .select(Note::as_select())
        .load(conn)?;
    Ok(notes.into_iter().map(ExportedNote::from).collect())
}

/// Imports everything in one transaction, keeping timestamps. Empty notes and notes whose
/// text already exists are skipped, so importing the same file twice adds nothing.
pub fn import_notes(
    conn: &mut SqliteConnection,
    notes: Vec<ExportedNote>,
) -> QueryResult<ImportResult> {
    conn.transaction(|conn| {
        let mut seen: HashSet<String> = note::table
            .select(note::body)
            .load::<String>(conn)?
            .into_iter()
            .collect();
        let mut result = ImportResult {
            imported: 0,
            skipped: 0,
        };

        for n in notes {
            if n.body.is_empty() || seen.contains(&n.body) {
                result.skipped += 1;
                continue;
            }
            diesel::insert_into(note::table)
                .values((
                    note::title.eq(title_of(&n.body)),
                    note::created_at.eq(n.created_at),
                    note::updated_at.eq(n.updated_at.unwrap_or(n.created_at)),
                    note::deleted_at.eq(n.deleted_at),
                    note::body.eq(&n.body),
                ))
                .execute(conn)?;
            seen.insert(n.body);
            result.imported += 1;
        }

        Ok(result)
    })
}
