import { invoke } from "@tauri-apps/api/core";

export interface IpInfo {
  ip: string;
  version: 4 | 6;
  source: string;
  checkedAt: string;
}

export interface HistoryEntry extends IpInfo {
  id: string;
  auto: boolean;
}

export interface WindowState {
  x: number;
  y: number;
  width: number;
  height: number;
  maximized: boolean;
}

export interface LookupResult {
  ip: string;
  version: 4 | 6;
  source: string;
  checkedAt: string;
}

export interface Settings {
  autoCheck: boolean;
  intervalMinutes: number;
  notifyOnChange: boolean;
  showAutoChecks: boolean;
  minimizeToTray: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  autoCheck: false,
  intervalMinutes: 10,
  notifyOnChange: true,
  showAutoChecks: true,
  minimizeToTray: true,
};

export const INTERVAL_OPTIONS = [5, 10, 15, 30, 60] as const;

export interface AutoCheckEvent {
  result: LookupResult;
  changed: boolean;
  previous: string | null;
}

export const AUTO_CHECK_EVENT = "ip://auto-check";

export const HISTORY_LIMIT = 50;

export const isTauri = "__TAURI_INTERNALS__" in window;

export const REPO_URL = "https://github.com/krisstianseyer/wan-ip-address";
export const REPO_LABEL = "github.com/krisstianseyer/wan-ip-address";

export async function openRepo(): Promise<void> {
  if (isTauri) {
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    await openUrl(REPO_URL);
    return;
  }
  window.open(REPO_URL, "_blank", "noopener");
}

const PREVIEW_KEY = "wan-ip-address:history";
const SETTINGS_KEY = "wan-ip-address:settings";

function previewHistory(): HistoryEntry[] {
  try {
    const raw = localStorage.getItem(PREVIEW_KEY);
    return raw ? (JSON.parse(raw) as HistoryEntry[]) : [];
  } catch {
    return [];
  }
}

function savePreview(entries: HistoryEntry[]): HistoryEntry[] {
  localStorage.setItem(PREVIEW_KEY, JSON.stringify(entries));
  return entries;
}

export async function lookupIp(): Promise<LookupResult> {
  if (isTauri) return invoke<LookupResult>("lookup_ip");

  const response = await fetch("https://api.ipify.org?format=json", {
    headers: { Accept: "application/json" },
  });
  if (!response.ok) throw new Error("Could not retrieve your WAN IP address.");
  const body = (await response.json()) as { ip: string };
  return {
    ip: body.ip,
    version: body.ip.includes(":") ? 6 : 4,
    source: "api.ipify.org",
    checkedAt: new Date().toISOString(),
  };
}

export async function loadHistory(): Promise<HistoryEntry[]> {
  if (isTauri) return invoke<HistoryEntry[]>("load_history");
  return previewHistory();
}

export async function recordIp(entry: LookupResult, auto = false): Promise<HistoryEntry[]> {
  if (isTauri) return invoke<HistoryEntry[]>("record_ip", { entry, auto });

  const history = previewHistory();
  const id = `${entry.checkedAt}-${entry.ip}`;
  if (history[0]?.ip === entry.ip) {
    Object.assign(history[0], entry, { id });
    if (!auto) history[0].auto = false;
  } else {
    history.unshift({ id, auto, ...entry });
  }
  return savePreview(history.slice(0, HISTORY_LIMIT));
}

export async function clearHistory(): Promise<void> {
  if (isTauri) return invoke("clear_history");
  localStorage.removeItem(PREVIEW_KEY);
}

export async function loadSettings(): Promise<Settings> {
  if (isTauri) return { ...DEFAULT_SETTINGS, ...(await invoke<Settings>("load_settings")) };
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (raw) return { ...DEFAULT_SETTINGS, ...(JSON.parse(raw) as Partial<Settings>) };
  } catch {
    /* fall back to defaults */
  }
  return { ...DEFAULT_SETTINGS };
}

export async function saveSettings(settings: Settings): Promise<Settings> {
  if (isTauri) return invoke<Settings>("save_settings", { settings });
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
  return settings;
}

export async function getWindowState(): Promise<WindowState | null> {
  if (!isTauri) return null;
  return invoke<WindowState | null>("get_window_state");
}

export async function hideToTray(): Promise<void> {
  if (!isTauri) return;
  await invoke("hide_to_tray");
}

export function relativeTime(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "";
  const seconds = Math.max(0, Math.round((Date.now() - then) / 1000));
  if (seconds < 45) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} hr ago`;
  const days = Math.round(hours / 24);
  if (days < 7) return `${days} d ago`;
  return new Date(then).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });
}

export function timeLabel(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return date.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}
