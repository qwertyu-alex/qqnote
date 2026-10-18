//! View ▸ Appearance: follow the system, or always use light or dark.
//!
//! The choice applies to the whole app, so the title bar and dialogs switch along with the
//! page (whose CSS follows `prefers-color-scheme`). It's saved in ~/.qqnote/settings.json.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::menu::{CheckMenuItem, CheckMenuItemBuilder, Submenu, SubmenuBuilder};
use tauri::{AppHandle, Runtime, Theme};

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

const ALL: [Appearance; 3] = [Appearance::System, Appearance::Light, Appearance::Dark];

impl Appearance {
    pub fn from_menu_id(id: &str) -> Option<Self> {
        ALL.into_iter()
            .find(|appearance| appearance.menu_id() == id)
    }

    fn menu_id(self) -> &'static str {
        match self {
            Appearance::System => "appearance_system",
            Appearance::Light => "appearance_light",
            Appearance::Dark => "appearance_dark",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Appearance::System => "System",
            Appearance::Light => "Light",
            Appearance::Dark => "Dark",
        }
    }

    fn theme(self) -> Option<Theme> {
        match self {
            Appearance::System => None,
            Appearance::Light => Some(Theme::Light),
            Appearance::Dark => Some(Theme::Dark),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    appearance: Appearance,
}

/// The View menu. Its checkmarks are set in `setup`, once the saved choice is loaded.
pub fn view_submenu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Submenu<R>> {
    let mut appearance = SubmenuBuilder::with_id(app, "appearance", "Appearance");
    for option in ALL {
        appearance = appearance
            .item(&CheckMenuItemBuilder::with_id(option.menu_id(), option.label()).build(app)?);
    }
    SubmenuBuilder::with_id(app, "view", "View")
        .item(&appearance.build()?)
        .build()
}

/// Applies the saved appearance. Runs on the main thread, so it lands before the first paint.
pub fn setup<R: Runtime>(app: &AppHandle<R>) {
    let appearance = load().appearance;
    app.set_theme(appearance.theme());
    sync_menu(app, appearance);
}

pub fn select<R: Runtime>(app: &AppHandle<R>, appearance: Appearance) {
    app.set_theme(appearance.theme());
    // Clicking only flipped the clicked item's checkmark.
    sync_menu(app, appearance);
    if let Err(e) = save(&Settings { appearance }) {
        eprintln!("Couldn't save the appearance: {}", e);
    }
}

fn sync_menu<R: Runtime>(app: &AppHandle<R>, selected: Appearance) {
    let submenu = app
        .menu()
        .and_then(|menu| menu.get("view"))
        .and_then(|view| view.as_submenu()?.get("appearance"))
        .and_then(|appearance| appearance.as_submenu().cloned());
    let Some(submenu) = submenu else {
        return;
    };
    for option in ALL {
        let item: Option<CheckMenuItem<R>> = submenu
            .get(option.menu_id())
            .and_then(|item| item.as_check_menuitem().cloned());
        if let Some(item) = item {
            let _ = item.set_checked(option == selected);
        }
    }
}

fn settings_path() -> Result<PathBuf, Error> {
    Ok(dirs::home_dir()
        .ok_or("couldn't find the home directory")?
        .join(".qqnote")
        .join("settings.json"))
}

/// The saved settings, or the defaults if there are none yet or they can't be read.
fn load() -> Settings {
    let read = || -> Result<Settings, Error> {
        let path = settings_path()?;
        if !path.exists() {
            return Ok(Settings::default());
        }
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    };
    read().unwrap_or_else(|e| {
        eprintln!("Couldn't read the settings: {}", e);
        Settings::default()
    })
}

fn save(settings: &Settings) -> Result<(), Error> {
    let path = settings_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}
