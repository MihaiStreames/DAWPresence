mod daw;
mod discord;
mod settings;

use daw::{format_cpu, format_memory, DawConfig, DawMonitor};
use discord::DiscordManager;
use settings::AppSettings;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

struct AppState {
    daw_monitor: Mutex<DawMonitor>,
    discord_manager: DiscordManager,
    settings: Mutex<AppSettings>,
    settings_path: std::path::PathBuf,
    last_connected: Mutex<bool>,
}

#[derive(Clone, serde::Serialize)]
struct StatusUpdate {
    daw_name: String,
    project_name: String,
    cpu_usage: String,
    ram_usage: String,
    is_connected: bool,
}

#[tauri::command]
fn get_status(state: tauri::State<Arc<AppState>>, app: AppHandle) -> StatusUpdate {
    let mut monitor = state.daw_monitor.lock().unwrap();

    let Some(daw_status) = monitor.get_running_daw() else {
        let _ = state.discord_manager.disconnect();
        update_tray_icon(&app, &state, false);
        return StatusUpdate {
            daw_name: String::new(),
            project_name: String::new(),
            cpu_usage: String::new(),
            ram_usage: String::new(),
            is_connected: false,
        };
    };

    let settings = state.settings.lock().unwrap();

    let details = if settings.hide_project_name {
        "Opening project:".to_string()
    } else {
        format!("Opening project: {}", daw_status.project_name)
    };

    let daw_state = if settings.hide_system_usage {
        daw_status.project_name.clone()
    } else {
        format!(
            "{} of CPU usage, {} of RAM usage",
            format_cpu(daw_status.cpu_usage),
            format_memory(daw_status.memory_usage)
        )
    };

    let is_connected = state
        .discord_manager
        .connect(&daw_status.client_id)
        .and_then(|_| {
            state
                .discord_manager
                .update_presence(&details, &daw_state, "icon", "DAWPresence")
        })
        .is_ok();

    drop(settings);
    update_tray_icon(&app, &state, is_connected);

    StatusUpdate {
        daw_name: daw_status.display_name,
        project_name: if state.settings.lock().unwrap().hide_project_name {
            "Hidden".to_string()
        } else {
            daw_status.project_name
        },
        cpu_usage: format_cpu(daw_status.cpu_usage),
        ram_usage: format_memory(daw_status.memory_usage),
        is_connected,
    }
}

#[tauri::command]
fn get_settings(state: tauri::State<Arc<AppState>>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn update_settings(
    state: tauri::State<Arc<AppState>>,
    new_settings: AppSettings,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings;
    settings.save(&state.settings_path)
}

#[tauri::command]
fn toggle_hide_project_name(state: tauri::State<Arc<AppState>>) -> Result<bool, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.hide_project_name = !settings.hide_project_name;
    settings.save(&state.settings_path)?;
    Ok(settings.hide_project_name)
}

#[tauri::command]
fn toggle_hide_system_usage(state: tauri::State<Arc<AppState>>) -> Result<bool, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.hide_system_usage = !settings.hide_system_usage;
    settings.save(&state.settings_path)?;
    Ok(settings.hide_system_usage)
}

#[tauri::command]
fn set_update_interval(state: tauri::State<Arc<AppState>>, interval: u64) -> Result<(), String> {
    if interval < 1000 {
        return Err("Interval must be at least 1000ms".to_string());
    }
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.update_interval = interval;
    settings.save(&state.settings_path)
}

const TRAY_ID: &str = "main-tray";

fn update_tray_icon(app: &AppHandle, state: &AppState, connected: bool) {
    let mut last = state.last_connected.lock().unwrap();
    if *last == connected {
        return;
    }
    *last = connected;

    let icon_name = if connected { "green.png" } else { "red.png" };
    let icon_path = app
        .path()
        .resource_dir()
        .map(|p| p.join("icons").join(icon_name))
        .ok();

    if let Some(path) = icon_path {
        if let Ok(icon) = Image::from_path(&path) {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_icon(Some(icon));
            }
        }
    }
}

fn load_daw_configs(app_handle: &AppHandle) -> Vec<DawConfig> {
    let resource_path = app_handle
        .path()
        .resource_dir()
        .ok()
        .map(|p| p.join("daws.json"));

    let app_data_path = app_handle
        .path()
        .app_data_dir()
        .ok()
        .map(|p| p.join("daws.json"));

    for path in [resource_path, app_data_path].into_iter().flatten() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(configs) = serde_json::from_str::<Vec<DawConfig>>(&content) {
                    return configs;
                }
            }
        }
    }

    Vec::new()
}

fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open = MenuItemBuilder::with_id("open", "Open Window").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&open, &quit]).build()?;

    let icon_path = app
        .path()
        .resource_dir()
        .map(|p| p.join("icons").join("red.png"))?;
    let icon = Image::from_path(&icon_path)?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("DAWPresence - Disconnected")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let daw_configs = load_daw_configs(&app_handle);

            let settings_path = app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("settings.json");

            if let Some(parent) = settings_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let settings = AppSettings::load(&settings_path);

            let state = Arc::new(AppState {
                daw_monitor: Mutex::new(DawMonitor::new(daw_configs)),
                discord_manager: DiscordManager::new(),
                settings: Mutex::new(settings),
                settings_path,
                last_connected: Mutex::new(false),
            });

            app.manage(state);
            setup_tray(&app_handle)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_settings,
            update_settings,
            toggle_hide_project_name,
            toggle_hide_system_usage,
            set_update_interval,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
