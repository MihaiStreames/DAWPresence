mod daw;
mod discord;
mod logging;
mod settings;

use daw::{format_cpu, format_memory, DawConfig, DawMonitor};
use discord::DiscordManager;
use settings::AppSettings;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
use tokio::time::sleep;

struct AppState {
    daw_monitor: Mutex<DawMonitor>,
    discord_manager: DiscordManager,
    settings: Mutex<AppSettings>,
    settings_path: std::path::PathBuf,
    last_connected: Mutex<bool>,
    status: Mutex<StatusUpdate>,
    tray_menu: TrayMenuItems,
    last_tray_status: Mutex<String>,
    last_daw_key: Mutex<Option<String>>,
}

struct TrayMenuItems {
    discord_status: tauri::menu::MenuItem<tauri::Wry>,
    hide_project_name: tauri::menu::MenuItem<tauri::Wry>,
    hide_system_usage: tauri::menu::MenuItem<tauri::Wry>,
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
fn get_status(state: tauri::State<Arc<AppState>>) -> StatusUpdate {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
fn get_settings(state: tauri::State<Arc<AppState>>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn update_settings(
    state: tauri::State<Arc<AppState>>,
    app: AppHandle,
    new_settings: AppSettings,
) -> Result<(), String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    *settings = new_settings;
    settings.save(&state.settings_path)?;
    drop(settings);

    update_tray_settings_menu(&state);
    let _ = app.emit_to("main", "settings-changed", ());
    logging::info("Settings updated");
    Ok(())
}

#[tauri::command]
fn toggle_hide_project_name(
    state: tauri::State<Arc<AppState>>,
    app: AppHandle,
) -> Result<bool, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.hide_project_name = !settings.hide_project_name;
    settings.save(&state.settings_path)?;
    let enabled = settings.hide_project_name;
    drop(settings);

    update_tray_settings_menu(&state);
    let _ = app.emit_to("main", "settings-changed", ());
    logging::info(format!("Hide project name: {enabled}"));
    Ok(enabled)
}

#[tauri::command]
fn toggle_hide_system_usage(
    state: tauri::State<Arc<AppState>>,
    app: AppHandle,
) -> Result<bool, String> {
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.hide_system_usage = !settings.hide_system_usage;
    settings.save(&state.settings_path)?;
    let enabled = settings.hide_system_usage;
    drop(settings);

    update_tray_settings_menu(&state);
    let _ = app.emit_to("main", "settings-changed", ());
    logging::info(format!("Hide system usage: {enabled}"));
    Ok(enabled)
}

#[tauri::command]
fn set_update_interval(
    state: tauri::State<Arc<AppState>>,
    app: AppHandle,
    interval: u64,
) -> Result<(), String> {
    if !(1000..=100_000_000).contains(&interval) {
        return Err("Update interval must be between 1000ms and 100,000,000ms".to_string());
    }
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    settings.update_interval = interval;
    settings.save(&state.settings_path)?;
    drop(settings);

    let _ = app.emit_to("main", "settings-changed", ());
    logging::info(format!("Update interval set to {interval}ms"));
    Ok(())
}

const TRAY_ID: &str = "main-tray";

fn tray_toggle_text(enabled: bool, label: &str) -> String {
    format!("[{}] {}", if enabled { "ON" } else { "OFF" }, label)
}

fn default_status() -> StatusUpdate {
    StatusUpdate {
        daw_name: "None".to_string(),
        project_name: "None".to_string(),
        cpu_usage: "Undefined".to_string(),
        ram_usage: "Undefined".to_string(),
        is_connected: false,
    }
}

fn update_tray_settings_menu(state: &AppState) {
    let settings = state.settings.lock().unwrap().clone();

    let _ = state
        .tray_menu
        .hide_project_name
        .set_text(tray_toggle_text(settings.hide_project_name, "Hide Project Name"));
    let _ = state
        .tray_menu
        .hide_system_usage
        .set_text(tray_toggle_text(settings.hide_system_usage, "Hide System Usage"));
}

fn update_tray_status(app: &AppHandle, state: &AppState, status: &str) {
    let mut last = state.last_tray_status.lock().unwrap();
    if *last == status {
        return;
    }
    *last = status.to_string();

    let tooltip = format!("DAWPresence - {}", status);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tooltip));
    }

    let _ = state.tray_menu.discord_status.set_text(status);
    logging::info(format!("Discord status: {}", status));
}

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

async fn run_update_loop(app: AppHandle, state: Arc<AppState>) {
    logging::info("Update loop started");
    loop {
        logging::trace("Tick");
        let settings = state.settings.lock().unwrap().clone();
        let interval = Duration::from_millis(settings.update_interval);

        let (daw_status, cpu_count) = {
            let mut monitor = state.daw_monitor.lock().unwrap();
            let cpu_count = monitor.cpu_count();
            let daw_status = monitor.get_running_daw(settings.hide_project_name);
            (daw_status, cpu_count)
        };

        match daw_status {
            None => {
                if state.last_daw_key.lock().unwrap().take().is_some() {
                    logging::info("No DAW detected");
                }
                let _ = state.discord_manager.disconnect();
                *state.status.lock().unwrap() = default_status();
                update_tray_icon(&app, &state, false);
                update_tray_status(&app, &state, "Open a DAW to begin displaying RPC");
            }
            Some(daw_status) => {
                let daw_key = format!(
                    "{}|{}|{}",
                    daw_status.display_name, daw_status.project_name, daw_status.client_id
                );
                let mut last_daw_key = state.last_daw_key.lock().unwrap();
                let daw_changed = last_daw_key
                    .as_ref()
                    .map(|k| k != &daw_key)
                    .unwrap_or(true);
                if daw_changed {
                    logging::info(format!(
                        "DAW detected: {} | Project: {} | PID: {}",
                        daw_status.display_name, daw_status.project_name, daw_status.pid
                    ));
                    *last_daw_key = Some(daw_key);
                }
                drop(last_daw_key);

                let cpu_usage = daw_status.cpu_usage / cpu_count as f32;
                let cpu_usage_str = format_cpu(cpu_usage);
                let ram_usage_str = format_memory(daw_status.memory_usage);

                let project_for_presence = if settings.hide_project_name {
                    "(hidden)".to_string()
                } else {
                    daw_status.project_name.clone()
                };

                let details = if ["None", "Untitled"].contains(&project_for_presence.as_str()) {
                    "Opening an untitled project".to_string()
                } else {
                    format!("Opening project: {}", project_for_presence)
                };

                let daw_state = if settings.hide_system_usage {
                    format!("Using {}", daw_status.display_name)
                } else {
                    let mut parts = Vec::new();
                    if !daw_status.hide_version && daw_status.version != "0.0.0" {
                        parts.push(format!("v{}", daw_status.version));
                    }
                    parts.push(format!("{} CPU", cpu_usage_str));
                    parts.push(format!("{} RAM", ram_usage_str));
                    parts.join(", ")
                };

                let large_text = format!("DAWPresence v{}", env!("CARGO_PKG_VERSION"));

                let discord_result = state
                    .discord_manager
                    .connect(&daw_status.client_id)
                    .and_then(|_| {
                        state
                            .discord_manager
                            .update_presence(&details, &daw_state, "icon", &large_text)
                    });

                let (is_connected, discord_status) = match discord_result {
                    Ok(()) => (true, "Connected to Discord".to_string()),
                    Err(error) => (false, format!("Connection failed: {}", error)),
                };

                let project_name = if settings.hide_project_name {
                    "(hidden)".to_string()
                } else {
                    daw_status.project_name.clone()
                };

                *state.status.lock().unwrap() = StatusUpdate {
                    daw_name: daw_status.display_name,
                    project_name,
                    cpu_usage: cpu_usage_str,
                    ram_usage: ram_usage_str,
                    is_connected,
                };

                update_tray_icon(&app, &state, is_connected);
                update_tray_status(&app, &state, &discord_status);
            }
        }

        sleep(interval).await;
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
                    logging::info(format!(
                        "Loaded {} DAWs from {}",
                        configs.len(),
                        path.display()
                    ));
                    return configs;
                }
            }
        }
    }

    logging::warn("No DAW configuration file found");
    Vec::new()
}

fn setup_tray(
    app: &AppHandle,
    settings: &AppSettings,
) -> Result<TrayMenuItems, Box<dyn std::error::Error>> {
    let version = MenuItemBuilder::with_id(
        "version",
        format!("DAWPresence v{}", env!("CARGO_PKG_VERSION")),
    )
    .enabled(false)
    .build(app)?;

    let discord_status = MenuItemBuilder::with_id(
        "discord_status",
        "Open a DAW to begin displaying RPC",
    )
    .enabled(false)
    .build(app)?;

    let open = MenuItemBuilder::with_id("open", "Open Window").build(app)?;

    let hide_project_name = MenuItemBuilder::with_id(
        "toggle_hide_project_name",
        tray_toggle_text(settings.hide_project_name, "Hide Project Name"),
    )
    .build(app)?;

    let hide_system_usage = MenuItemBuilder::with_id(
        "toggle_hide_system_usage",
        tray_toggle_text(settings.hide_system_usage, "Hide System Usage"),
    )
    .build(app)?;

    let update_interval = MenuItemBuilder::with_id("set_update_interval", "Set Update Interval")
        .build(app)?;

    let settings_menu = SubmenuBuilder::new(app, "Settings")
        .items(&[&hide_project_name, &hide_system_usage, &update_interval])
        .build()?;

    let exit = MenuItemBuilder::with_id("quit", "Exit").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&version, &discord_status])
        .separator()
        .item(&open)
        .item(&settings_menu)
        .separator()
        .item(&exit)
        .build()?;

    let icon_path = app
        .path()
        .resource_dir()
        .map(|p| p.join("icons").join("red.png"))?;
    let icon = Image::from_path(&icon_path)?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("DAWPresence - Open a DAW to begin displaying RPC")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "toggle_hide_project_name" => {
                let state = app.state::<Arc<AppState>>().inner();
                if let Ok(mut settings) = state.settings.lock() {
                    settings.hide_project_name = !settings.hide_project_name;
                    let _ = settings.save(&state.settings_path);
                    logging::info(format!("Hide project name: {}", settings.hide_project_name));
                }
                update_tray_settings_menu(state);
                let _ = app.emit_to("main", "settings-changed", ());
            }
            "toggle_hide_system_usage" => {
                let state = app.state::<Arc<AppState>>().inner();
                if let Ok(mut settings) = state.settings.lock() {
                    settings.hide_system_usage = !settings.hide_system_usage;
                    let _ = settings.save(&state.settings_path);
                    logging::info(format!("Hide system usage: {}", settings.hide_system_usage));
                }
                update_tray_settings_menu(state);
                let _ = app.emit_to("main", "settings-changed", ());
            }
            "set_update_interval" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let _ = app.emit_to("main", "open-update-interval", ());
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

    Ok(TrayMenuItems {
        discord_status,
        hide_project_name,
        hide_system_usage,
    })
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

            logging::info(format!(
                "Settings loaded (update_interval={}ms)",
                settings.update_interval
            ));
            let tray_menu = setup_tray(&app_handle, &settings)?;

            let state = Arc::new(AppState {
                daw_monitor: Mutex::new(DawMonitor::new(daw_configs)),
                discord_manager: DiscordManager::new(),
                settings: Mutex::new(settings),
                settings_path,
                last_connected: Mutex::new(false),
                status: Mutex::new(default_status()),
                tray_menu,
                last_tray_status: Mutex::new(String::new()),
                last_daw_key: Mutex::new(None),
            });

            app.manage(state.clone());

            let app_handle = app_handle.clone();
            let state = state.clone();
            tauri::async_runtime::spawn(async move {
                run_update_loop(app_handle, state).await;
            });

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
