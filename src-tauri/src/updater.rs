//! qqnote keeps itself up to date from its GitHub releases.
//!
//! Release builds check shortly after launch and then every few hours. A newer version is
//! downloaded and installed in the background, and the running copy keeps going until it
//! restarts: qqnote ▸ Restart to Update and the status bar offer that, and otherwise the next
//! launch picks it up. Updates are signed with the key whose public half is in
//! tauri.conf.json, and anything else is refused.

use std::sync::Mutex;
use std::time::Duration;
use tauri::menu::MenuItem;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

use crate::launcher;

const FIRST_CHECK_DELAY: Duration = Duration::from_secs(10);
const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub struct UpdateState {
    /// Held while checking or installing, so a check from the menu waits for a background one.
    busy: Mutex<()>,
    /// The version that's installed and waiting for a restart.
    ready: Mutex<Option<String>>,
}

impl UpdateState {
    pub fn ready(&self) -> Option<String> {
        self.ready.lock().unwrap().clone()
    }
}

/// The updater replaces the app bundle the executable is in. A dev build (or any executable
/// outside qqnote.app) isn't in one, and it would replace the build folder instead.
fn can_update() -> bool {
    std::env::current_exe()
        .is_ok_and(|exe| exe.to_string_lossy().contains(".app/Contents/MacOS/"))
}

pub fn setup<R: Runtime>(app: &AppHandle<R>) {
    if !can_update() {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST_CHECK_DELAY);
        loop {
            match install_update(&app) {
                // Nothing more to do until qqnote restarts into it.
                Ok(Some(_)) => return,
                Ok(None) => {}
                Err(e) => eprintln!("Couldn't update qqnote: {}", e),
            }
            std::thread::sleep(CHECK_INTERVAL);
        }
    });
}

/// qqnote ▸ Check for Updates…, which becomes Restart to Update once one is installed.
pub fn menu_clicked<R: Runtime>(app: &AppHandle<R>) {
    if app.state::<UpdateState>().ready().is_some() {
        crate::request_restart(app);
        return;
    }
    if !can_update() {
        app.dialog()
            .message("Only qqnote.app updates itself, not a development build.")
            .title("qqnote")
            .show(|_| {});
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || match install_update(&app) {
        Ok(Some(version)) => offer_restart(&app, &version),
        Ok(None) => {
            app.dialog()
                .message(format!(
                    "Version {} is the newest version.",
                    app.package_info().version
                ))
                .title("qqnote is up to date")
                .show(|_| {});
        }
        Err(e) => {
            app.dialog()
                .message(e)
                .title("Couldn't check for updates")
                .kind(MessageDialogKind::Error)
                .show(|_| {});
        }
    });
}

/// Installs the newest version if there's one. Returns the version that's waiting for a
/// restart, if any. Blocks, so it runs off the main thread.
fn install_update<R: Runtime>(app: &AppHandle<R>) -> Result<Option<String>, String> {
    let state = app.state::<UpdateState>();
    let _busy = state.busy.lock().unwrap();
    if let Some(version) = state.ready() {
        return Ok(Some(version));
    }

    let installed = tauri::async_runtime::block_on(async {
        let Some(update) = app.updater()?.check().await? else {
            return Ok(None);
        };
        update.download_and_install(|_, _| {}, || {}).await?;
        Ok::<_, tauri_plugin_updater::Error>(Some(update.version))
    })
    .map_err(|e| e.to_string())?;

    if let Some(version) = &installed {
        *state.ready.lock().unwrap() = Some(version.clone());
        if let Some(item) = menu_item(app) {
            let _ = item.set_text("Restart to Update");
        }
        let _ = app.emit("update-ready", version);
    }
    Ok(installed)
}

fn offer_restart<R: Runtime>(app: &AppHandle<R>, version: &str) {
    let restart_app = app.clone();
    app.dialog()
        .message(format!(
            "qqnote {} is installed. Restart now to start using it? Your open note is saved first.",
            version
        ))
        .title("Update installed")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Restart".into(),
            "Later".into(),
        ))
        .show(move |restart| {
            if restart {
                crate::request_restart(&restart_app);
            }
        });
}

fn menu_item<R: Runtime>(app: &AppHandle<R>) -> Option<MenuItem<R>> {
    app.menu()
        .and_then(|menu| menu.get("file"))
        .and_then(|file| file.as_submenu()?.get("check_for_updates"))
        .and_then(|item| item.as_menuitem().cloned())
}

/// Starts the installed version in place of this one. Unlike `AppHandle::restart`, it gives
/// Caps Lock back first (restarting skips `RunEvent::Exit`), and the new copy shows its
/// window even if this one started hidden at login.
pub fn restart<R: Runtime>(app: &AppHandle<R>) {
    let handle = app.clone();
    let result = app.run_on_main_thread(move || {
        launcher::teardown();
        handle.cleanup_before_exit();
        let mut env = handle.env();
        env.args_os.retain(|arg| arg != launcher::BACKGROUND_ARG);
        tauri::process::restart(&env);
    });
    if result.is_err() {
        app.exit(0);
    }
}
