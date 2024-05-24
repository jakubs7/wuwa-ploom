#![windows_subsystem = "windows"]

use rusqlite::{params, Connection};
use serde_json::{Value, json};
use std::fs;
use thiserror::Error;
use eframe::{egui, App, CreationContext, Frame, NativeOptions};
use egui::CentralPanel;
use rfd::FileDialog;
use winapi::um::wincon::SetConsoleTitleW;
use winapi::shared::windef::{HWND, HICON};
use winapi::um::winuser::{
    FindWindowW, GetWindowLongW, SetWindowLongW, SetClassLongPtrW, LoadImageW,
    GWL_STYLE, WS_SYSMENU, WS_MINIMIZEBOX, GCLP_HICON, GCLP_HICONSM, LR_DEFAULTSIZE, LR_LOADFROMFILE, IMAGE_ICON,
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use winreg::enums::*;
use winreg::RegKey;
use serde::de::Error as SerdeError;

#[derive(Error, Debug)]
enum MyError {
    #[error("Database error: {0}")]
    RusqliteError(#[from] rusqlite::Error),
    #[error("JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Registry error: Could not access the registry key or value.")]
    RegistryError,
    #[error("File not found or inaccessible: {0}")]
    FileNotFoundError(String),
}

type Result<T> = std::result::Result<T, MyError>;

fn set_console_title(title: &str) {
    let wide: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
    unsafe {
        SetConsoleTitleW(wide.as_ptr());
    }
}

fn get_hwnd(title: &str) -> HWND {
    let wide_title: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    unsafe {
        FindWindowW(ptr::null(), wide_title.as_ptr())
    }
}

fn remove_window_icon(hwnd: HWND) {
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
    let new_style = style & !(WS_SYSMENU as i32 | WS_MINIMIZEBOX as i32);
    unsafe {
        SetWindowLongW(hwnd, GWL_STYLE, new_style);
    }
}

fn load_icon(path: &str) -> HICON {
    let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
    unsafe {
        LoadImageW(
            ptr::null_mut(),
            wide_path.as_ptr(),
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_LOADFROMFILE,
        ) as HICON
    }
}

fn set_window_icon(hwnd: HWND, icon: HICON) {
    unsafe {
        SetClassLongPtrW(hwnd, GCLP_HICON as i32, icon as isize);
        SetClassLongPtrW(hwnd, GCLP_HICONSM as i32, icon as isize);
    }
}

fn get_game_install_path() -> Result<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let game_key_path = "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\KRInstall Wuthering Waves Overseas";
    let game_key = hklm.open_subkey(game_key_path).map_err(|_| MyError::RegistryError)?;
    let install_path: String = game_key.get_value("InstallPath").map_err(|_| MyError::RegistryError)?;
    let full_path = format!("{}\\Wuthering Waves Game\\Client\\Saved\\LocalStorage\\LocalStorage.db", install_path);
    Ok(full_path)
}

fn file_exists(path: &str) -> Result<()> {
    if fs::metadata(path).is_err() {
        Err(MyError::FileNotFoundError(path.to_string()))
    } else {
        Ok(())
    }
}

fn read_game_quality_setting(conn: &Connection) -> Result<Value> {
    let mut stmt = conn.prepare("SELECT value FROM LocalStorage WHERE key = 'GameQualitySetting';")?;
    let mut rows = stmt.query([])?;

    let game_quality_setting_json: String = rows.next()?.ok_or(rusqlite::Error::QueryReturnedNoRows)?.get(0)?;
    let game_quality_setting: Value = serde_json::from_str(&game_quality_setting_json)?;
    Ok(game_quality_setting)
}

fn read_current_fps_setting(db_path: &str) -> Result<i64> {
    file_exists(db_path)?;
    let conn = Connection::open(db_path)?;
    let game_quality_setting = read_game_quality_setting(&conn)?;
    let fps_setting = game_quality_setting["KeyCustomFrameRate"]
        .as_i64()
        .ok_or_else(|| MyError::SerdeJsonError(SerdeError::custom("KeyCustomFrameRate not found or not an integer")))?;
    Ok(fps_setting)
}

fn update_game_quality_setting(conn: &Connection, game_quality_setting: Value) -> Result<()> {
    let updated_game_quality_setting_json = game_quality_setting.to_string();
    conn.execute(
        "UPDATE LocalStorage SET value = ?1 WHERE key = 'GameQualitySetting';",
        params![updated_game_quality_setting_json],
    )?;
    Ok(())
}

fn unlock_fps(db_path: &str) -> Result<String> {
    file_exists(db_path)?;
    let conn = Connection::open(db_path)?;
    let mut game_quality_setting = read_game_quality_setting(&conn)?;

    if game_quality_setting["KeyCustomFrameRate"] == json!(120) {
        return Ok("FPS is already set to 120. No need to patch.".into());
    }

    game_quality_setting["KeyCustomFrameRate"] = json!(120);
    update_game_quality_setting(&conn, game_quality_setting)?;

    Ok("FPS successfully unlocked to 120!".into())
}

fn unlock_fps_165(db_path: &str) -> Result<String> {
    file_exists(db_path)?;
    let conn = Connection::open(db_path)?;
    let mut game_quality_setting = read_game_quality_setting(&conn)?;

    if game_quality_setting["KeyCustomFrameRate"] == json!(165) {
        return Ok("FPS is already set to 165. No need to patch.".into());
    }

    game_quality_setting["KeyCustomFrameRate"] = json!(165);
    update_game_quality_setting(&conn, game_quality_setting)?;

    Ok("FPS successfully unlocked to 165!".into())
}

struct FPSUnlockerApp {
    db_path: String,
    status: String,
    current_fps: Option<i64>,
}

impl Default for FPSUnlockerApp {
    fn default() -> Self {
        Self {
            db_path: String::new(),
            status: String::new(),
            current_fps: None,
        }
    }
}

const APP_TITLE: &str = "WuWa Ploom 120 & 165 FPS Unlock";
const INSTRUCTIONS: &str = "
1) Check and set your FPS limit to 60, then close your game.
2) Do not touch FPS or VSync options in-game.
3) You can either automatically find it or browse and choose the file.
";

impl App for FPSUnlockerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(APP_TITLE);
            });
            ui.separator();
            ui.add_space(10.0);
            ui.label("Made by abellio");
            ui.horizontal(|ui| {
                ui.label("Github:");
                ui.hyperlink("https://github.com/jakubs7");
                ui.label("");
            });
            ui.add_space(10.0);
            ui.label("Support my Gacha addiction:");
            ui.horizontal(|ui| {
                ui.label("ko-fi:");
                ui.hyperlink("https://ko-fi.com/abellio");
                ui.label("");
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label("Steps:");
            ui.label(INSTRUCTIONS);
            ui.separator();
            ui.label("Select the SQLite database file:");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Locate Configuration File").clicked() {
                    match get_game_install_path() {
                        Ok(path) => {
                            self.db_path = path;
                            match read_current_fps_setting(&self.db_path) {
                                Ok(fps) => self.current_fps = Some(fps),
                                Err(err) => self.status = format!("Error reading FPS setting: {}", err),
                            }
                        },
                        Err(err) => self.status = format!("Error locating game: {}", err),
                    }
                }
            
                if ui.button("Browse for Configuration File").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        self.db_path = path.display().to_string();
                        match read_current_fps_setting(&self.db_path) {
                            Ok(fps) => self.current_fps = Some(fps),
                            Err(err) => self.status = format!("Error reading FPS setting: {}", err),
                        }
                    }
                }

                if ui.button("Set FPS to 120").clicked() {
                    match unlock_fps(&self.db_path) {
                        Ok(message) => self.status = message,
                        Err(err) => self.status = format!("Error: {}", err),
                    }
                }

                if ui.button("Set FPS to 165").clicked() {
                    match unlock_fps_165(&self.db_path) {
                        Ok(message) => self.status = message,
                        Err(err) => self.status = format!("Error: {}", err),
                    }
                }
            });
            ui.add_space(10.0);
            ui.label(&self.db_path);

            if let Some(fps) = self.current_fps {
                ui.separator();
                ui.label("Current FPS Setting:");
                ui.label(format!("KeyCustomFrameRate: {}", fps));
                if fps == 120 {
                    ui.label("FPS is already set to 120. No need to patch.");
                }
                if fps == 165 {
                    ui.label("FPS is already set to 165. No need to patch.");
                }
            }
            ui.add_space(10.0);
            ui.label(&self.status);
        });
    }
}

fn main() {
    set_console_title("WuWa Ploom FPS Unlock");

    let native_options = NativeOptions::default();
    eframe::run_native(
        "WuWa Ploom Tools",
        native_options,
        Box::new(|_cc: &CreationContext| Box::new(FPSUnlockerApp::default())),
    );

    let hwnd = get_hwnd("WuWa Ploom Tools");
    if !hwnd.is_null() {
        let icon = load_icon("misc/ploom.ico");
        if icon.is_null() {
            eprintln!("Failed to load icon.");
        } else {
            set_window_icon(hwnd, icon);
        }
        remove_window_icon(hwnd);
    }
}
