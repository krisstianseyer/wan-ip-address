use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WindowEvent};

const SOURCES: [&str; 2] = [
    "https://api.ipify.org?format=json",
    "https://api64.ipify.org?format=json",
];

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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Persisted {
    history: Vec<HistoryEntry>,
    window: Option<WindowState>,
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

#[tauri::command]
async fn lookup_ip() -> Result<LookupResult, String> {
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
fn load_history(app: AppHandle) -> Vec<HistoryEntry> {
    read_state(&app).history
}

#[tauri::command]
fn record_ip(app: AppHandle, entry: LookupResult) -> Result<Vec<HistoryEntry>, String> {
    let mut state = read_state(&app);
    let id = format!("{}-{}", entry.checked_at, entry.ip);

    if let Some(latest) = state.history.first_mut() {
        if latest.ip == entry.ip {
            latest.id = id;
            latest.checked_at = entry.checked_at;
            latest.source = entry.source;
            latest.version = entry.version;
            write_state(&app, &state)?;
            return Ok(state.history);
        }
    }

    state.history.insert(
        0,
        HistoryEntry {
            id,
            ip: entry.ip,
            version: entry.version,
            source: entry.source,
            checked_at: entry.checked_at,
        },
    );
    state.history.truncate(50);
    write_state(&app, &state)?;
    Ok(state.history)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            lookup_ip,
            load_history,
            record_ip,
            clear_history,
            get_window_state
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                let tracked = window.clone();
                window.on_window_event(move |event| {
                    if !matches!(event, WindowEvent::CloseRequested { .. }) {
                        return;
                    }
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
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running WAN IP Address");
}
