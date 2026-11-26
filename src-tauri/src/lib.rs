use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};
use tauri_plugin_notification::NotificationExt;

const SOURCES: [&str; 2] = [
    "https://api.ipify.org?format=json",
    "https://api64.ipify.org?format=json",
];

const HISTORY_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupResult {
    pub ip: String,
    pub version: u8,
    pub source: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub ip: String,
    pub version: u8,
    pub source: String,
    pub checked_at: String,
    #[serde(default)]
    pub auto: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub auto_check: bool,
    pub interval_minutes: u32,
    pub notify_on_change: bool,
    pub show_auto_checks: bool,
    #[serde(default = "default_minimize_to_tray")]
    pub minimize_to_tray: bool,
}

fn default_minimize_to_tray() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_check: false,
            interval_minutes: 10,
            notify_on_change: true,
            show_auto_checks: true,
            minimize_to_tray: true,
        }
    }
}

impl Settings {
    fn normalized(self) -> Self {
        Self {
            interval_minutes: self.interval_minutes.clamp(1, 1440),
            ..self
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Persisted {
    history: Vec<HistoryEntry>,
    window: Option<WindowState>,
    settings: Settings,
}

struct AutoCheckSignal(tokio::sync::Notify);

impl Default for AutoCheckSignal {
    fn default() -> Self {
        Self(tokio::sync::Notify::new())
    }
}

#[derive(Default)]
struct TrayRestore(AtomicBool);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutoCheckEvent {
    result: LookupResult,
    changed: bool,
    previous: Option<String>,
}

#[derive(Deserialize)]
struct IpifyResponse {
    ip: String,
}

fn state_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("state.json"))
        .map_err(|err| err.to_string())
}

fn read_state(app: &AppHandle) -> Persisted {
    let Ok(path) = state_path(app) else {
        return Persisted::default();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return Persisted::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn write_state(app: &AppHandle, state: &Persisted) -> Result<(), String> {
    let path = state_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|err| err.to_string())?;
    fs::write(path, text).map_err(|err| err.to_string())
}

fn source_host(url: &str) -> String {
    url.strip_prefix("https://")
        .unwrap_or(url)
        .split(['/', '?'])
        .next()
        .unwrap_or(url)
        .to_string()
}

async fn fetch_public_ip() -> Result<LookupResult, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(format!("wan-ip-address/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|err| err.to_string())?;

    let mut errors: Vec<String> = Vec::new();

    for url in SOURCES {
        let attempt = async {
            let response = client.get(url).send().await.map_err(|err| err.to_string())?;
            let body: IpifyResponse = response.json().await.map_err(|err| err.to_string())?;
            let parsed: std::net::IpAddr = body.ip.parse().map_err(|_| "invalid address".to_string())?;
            Ok::<LookupResult, String>(LookupResult {
                ip: body.ip,
                version: if parsed.is_ipv4() { 4 } else { 6 },
                source: source_host(url),
                checked_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            })
        }
        .await;

        match attempt {
            Ok(result) => return Ok(result),
            Err(err) => errors.push(format!("{}: {}", source_host(url), err)),
        }
    }

    Err(format!(
        "Could not retrieve your WAN IP address.\n{}",
        errors.join("\n")
    ))
}

#[tauri::command]
async fn lookup_ip() -> Result<LookupResult, String> {
    fetch_public_ip().await
}

fn push_entry(history: &mut Vec<HistoryEntry>, entry: &LookupResult, auto: bool) {
    let id = format!("{}-{}", entry.checked_at, entry.ip);

    if let Some(latest) = history.first_mut() {
        if latest.ip == entry.ip {
            latest.id = id;
            latest.checked_at = entry.checked_at.clone();
            latest.source = entry.source.clone();
            latest.version = entry.version;
            if !auto {
                latest.auto = false;
            }
            return;
        }
    }

    history.insert(
        0,
        HistoryEntry {
            id,
            ip: entry.ip.clone(),
            version: entry.version,
            source: entry.source.clone(),
            checked_at: entry.checked_at.clone(),
            auto,
        },
    );
    history.truncate(HISTORY_LIMIT);
}

#[tauri::command]
fn load_history(app: AppHandle) -> Vec<HistoryEntry> {
    read_state(&app).history
}

#[tauri::command]
fn record_ip(app: AppHandle, entry: LookupResult, auto: bool) -> Result<Vec<HistoryEntry>, String> {
    let mut state = read_state(&app);
    push_entry(&mut state.history, &entry, auto);
    write_state(&app, &state)?;
    Ok(state.history)
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Settings {
    read_state(&app).settings.normalized()
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<Settings, String> {
    let settings = settings.normalized();
    let mut state = read_state(&app);
    state.settings = settings.clone();
    write_state(&app, &state)?;
    app.state::<AutoCheckSignal>().0.notify_one();
    Ok(settings)
}

async fn auto_check_now(app: &AppHandle) {
    let Ok(result) = fetch_public_ip().await else {
        return;
    };

    let mut state = read_state(app);
    let previous = state.history.first().map(|entry| entry.ip.clone());
    let changed = previous.as_deref().is_some_and(|ip| ip != result.ip);

    push_entry(&mut state.history, &result, true);
    let _ = write_state(app, &state);

    if changed && state.settings.notify_on_change {
        let body = match &previous {
            Some(old) => format!("{}  →  {}", old, result.ip),
            None => result.ip.clone(),
        };
        let _ = app
            .notification()
            .builder()
            .title("WAN IP address changed")
            .body(body)
            .show();
    }

    let _ = app.emit(
        "ip://auto-check",
        AutoCheckEvent {
            result,
            changed,
            previous,
        },
    );
}

fn start_auto_check(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let settings = read_state(&app).settings;
            let wait = if settings.auto_check {
                Duration::from_secs(settings.interval_minutes.max(1) as u64 * 60)
            } else {
                Duration::from_secs(3600)
            };

            let signal = app.state::<AutoCheckSignal>();
            tokio::select! {
                _ = tokio::time::sleep(wait) => {
                    if read_state(&app).settings.auto_check {
                        auto_check_now(&app).await;
                    }
                }
                _ = signal.0.notified() => {}
            }
        }
    });
}

#[tauri::command]
fn clear_history(app: AppHandle) -> Result<(), String> {
    let mut state = read_state(&app);
    state.history.clear();
    write_state(&app, &state)
}

#[tauri::command]
fn get_window_state(app: AppHandle) -> Option<WindowState> {
    read_state(&app).window
}

fn show_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    app.state::<TrayRestore>().0.store(true, Ordering::SeqCst);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(800)).await;
        handle.state::<TrayRestore>().0.store(false, Ordering::SeqCst);
    });
}

fn toggle_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let visible = window.is_visible().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(false);

    if visible && !minimized {
        let _ = window.hide();
    } else {
        show_main_window(app);
    }
}

#[tauri::command]
fn hide_to_tray(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window is not available".to_string())?;
    window.hide().map_err(|err| err.to_string())
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show WAN IP Address", true, None::<&str>)?;
    let check = MenuItem::with_id(app, "check", "Check now", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &check, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("WAN IP Address")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "check" => {
                show_main_window(app);
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    auto_check_now(&handle).await;
                });
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
                toggle_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            lookup_ip,
            load_history,
            record_ip,
            clear_history,
            get_window_state,
            load_settings,
            save_settings,
            hide_to_tray
        ])
        .setup(|app| {
            app.manage(AutoCheckSignal::default());
            app.manage(TrayRestore::default());
            start_auto_check(app.handle().clone());
            setup_tray(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                let tracked = window.clone();
                window.on_window_event(move |event| {
                    match event {
                        WindowEvent::Resized(_) => {
                            if tracked.is_minimized().unwrap_or(false)
                                && read_state(&handle).settings.minimize_to_tray
                                && !handle.state::<TrayRestore>().0.load(Ordering::SeqCst)
                            {
                                let _ = tracked.hide();
                            }
                        }
                        WindowEvent::CloseRequested { .. } => {
                            let maximized = tracked.is_maximized().unwrap_or(false);
                            let mut state = read_state(&handle);

                            if maximized {
                                if let Some(window_state) = state.window.as_mut() {
                                    window_state.maximized = true;
                                } else {
                                    state.window = Some(WindowState {
                                        x: 120,
                                        y: 120,
                                        width: 420,
                                        height: 560,
                                        maximized: true,
                                    });
                                }
                            } else if let (Ok(position), Ok(size)) =
                                (tracked.outer_position(), tracked.inner_size())
                            {
                                state.window = Some(WindowState {
                                    x: position.x,
                                    y: position.y,
                                    width: size.width,
                                    height: size.height,
                                    maximized: false,
                                });
                            }

                            let _ = write_state(&handle, &state);
                        }
                        _ => {}
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running WAN IP Address");
}
