<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { PhysicalPosition, PhysicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import TitleBar from "./components/TitleBar.vue";
import HistoryList from "./components/HistoryList.vue";
import SettingsPanel from "./components/SettingsPanel.vue";
import {
  AUTO_CHECK_EVENT,
  clearHistory,
  DEFAULT_SETTINGS,
  getWindowState,
  HISTORY_LIMIT,
  isTauri,
  loadHistory,
  loadSettings,
  lookupIp,
  openRepo,
  recordIp,
  REPO_LABEL,
  saveSettings,
  timeLabel,
  type AutoCheckEvent,
  type HistoryEntry,
  type LookupResult,
  type Settings,
} from "./lib/api";

const ip = ref<LookupResult | null>(null);
const history = ref<HistoryEntry[]>([]);
const settings = ref<Settings>({ ...DEFAULT_SETTINGS });
const settingsOpen = ref(false);
const savingSettings = ref(false);
const changeNotice = ref<{ previous: string | null; current: string } | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const copiedMain = ref(false);
const copiedId = ref<string | null>(null);
const confirmingClear = ref(false);

const win = isTauri ? getCurrentWindow() : null;
let copyTimer: number | undefined;
let clearTimer: number | undefined;
let changeTimer: number | undefined;
let unlistenAuto: UnlistenFn | null = null;
let saveToken = 0;

const visibleHistory = computed(() =>
  settings.value.showAutoChecks ? history.value : history.value.filter((entry) => !entry.auto),
);

const hiddenAutoCount = computed(() => history.value.length - visibleHistory.value.length);

const ipSize = computed(() => {
  const length = ip.value?.ip.length ?? 0;
  if (length > 30) return "text-[19px] leading-[1.45]";
  if (length > 18) return "text-[24px] leading-[1.35]";
  return "text-[30px] leading-[1.3]";
});

const statusLabel = computed(() => {
  if (loading.value) return "Checking…";
  if (error.value) return "Lookup failed";
  return "Connected";
});

function prettyError(message: unknown): string {
  const text = String(message ?? "Something went wrong.").replace(/^Error:\s*/, "");
  const [first] = text.split("\n");
  return first || "Could not retrieve your WAN IP address.";
}

async function check(): Promise<void> {
  if (loading.value) return
  loading.value = true;
  error.value = null;

  try {
    const result = await lookupIp();
    ip.value = result;
    history.value = await recordIp(result);
  } catch (err) {
    error.value = prettyError(err);
  } finally {
    loading.value = false;
  }
}

function showChange(previous: string | null, current: string): void {
  changeNotice.value = { previous, current };
  window.clearTimeout(changeTimer);
  changeTimer = window.setTimeout(() => (changeNotice.value = null), 12_000);
}

function dismissChange(): void {
  changeNotice.value = null;
  window.clearTimeout(changeTimer);
}

function openSettings(): void {
  settingsOpen.value = true;
}

function closeSettings(): void {
  settingsOpen.value = false;
}

async function updateSettings(patch: Partial<Settings>): Promise<void> {
  const next = { ...settings.value, ...patch };
  settings.value = next;
  const token = ++saveToken;
  savingSettings.value = true;
  try {
    const saved = await saveSettings(next);
    if (token === saveToken) settings.value = saved;
  } catch {
    /* keep the local preference if the state file cannot be written */
  } finally {
    if (token === saveToken) savingSettings.value = false;
  }
}

async function onAutoCheck(event: { payload: AutoCheckEvent }): Promise<void> {
  ip.value = event.payload.result;
  error.value = null;
  try {
    history.value = await loadHistory();
  } catch {
    /* keep the current list if the state file cannot be read */
  }
  if (event.payload.changed) showChange(event.payload.previous, event.payload.result.ip);
}

async function copy(text: string): Promise<boolean> {
  try {
    if (isTauri) {
      await writeText(text);
    } else {
      await navigator.clipboard.writeText(text);
    }
    return true;
  } catch {
    return false;
  }
}

async function copyCurrent(): Promise<void> {
  if (!ip.value) return;
  if (!(await copy(ip.value.ip))) return;
  copiedMain.value = true;
  window.clearTimeout(copyTimer);
  copyTimer = window.setTimeout(() => (copiedMain.value = false), 1600);
}

async function copyEntry(entry: HistoryEntry): Promise<void> {
  if (!(await copy(entry.ip))) return;
  copiedId.value = entry.id;
  window.clearTimeout(copyTimer);
  copyTimer = window.setTimeout(() => (copiedId.value = null), 1600);
}

function askClear(): void {
  if (confirmingClear.value) {
    void clearAll();
    return;
  }
  confirmingClear.value = true;
  window.clearTimeout(clearTimer);
  clearTimer = window.setTimeout(() => (confirmingClear.value = false), 2600);
}

async function clearAll(): Promise<void> {
  confirmingClear.value = false;
  window.clearTimeout(clearTimer);
  try {
    await clearHistory();
    history.value = [];
  } catch {
    /* keep the list if the state file cannot be written */
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape" && settingsOpen.value) {
    closeSettings();
    return;
  }
  const mod = event.ctrlKey || event.metaKey;
  if (mod && event.key.toLowerCase() === "r") {
    event.preventDefault();
    void check();
  }
  if (mod && event.shiftKey && event.key.toLowerCase() === "c") {
    event.preventDefault();
    void copyCurrent();
  }
}

onMounted(async () => {
  window.addEventListener("keydown", onKeydown);

  if (win) {
    try {
      const saved = await getWindowState();
      if (saved) {
        await win.setSize(new PhysicalSize(saved.width, saved.height));
        await win.setPosition(new PhysicalPosition(saved.x, saved.y));
        if (saved.maximized) await win.maximize();
      }
    } catch {
      /* fall back to the centered default size */
    } finally {
      await win.show();
    }
  }

  try {
    settings.value = await loadSettings();
  } catch {
    /* keep defaults */
  }

  try {
    history.value = await loadHistory();
  } catch {
    /* start with an empty list */
  }

  if (isTauri) {
    try {
      unlistenAuto = await listen<AutoCheckEvent>(AUTO_CHECK_EVENT, (event) => {
        void onAutoCheck(event);
      });
    } catch {
      /* auto-check updates still land in the state file */
    }
  }

  void check();
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  window.clearTimeout(copyTimer);
  window.clearTimeout(clearTimer);
  window.clearTimeout(changeTimer);
  unlistenAuto?.();
});
</script>

<template>
  <div
    class="flex h-full flex-col overflow-hidden rounded-[12px] border border-[#E1E4E9] bg-shell"
  >
    <TitleBar />

    <main
      class="mx-1.5 mb-1.5 flex min-h-0 flex-1 flex-col rounded-[20px] border border-line bg-panel px-5 pb-3.5 pt-4"
    >
      <template v-if="!settingsOpen">
      <div class="flex items-center justify-between gap-3">
        <div class="flex min-w-0 items-center gap-1.5 text-[11.5px]">
          <span
            class="h-1.5 w-1.5 shrink-0 rounded-full"
            :class="
              error
                ? 'bg-danger'
                : loading
                  ? 'bg-[#c2c7cf]'
                  : 'bg-positive'
            "
          />
          <span class="text-muted">{{ statusLabel }}</span>
          <template v-if="ip && !error">
            <span class="text-[#c9cdd4]">·</span>
            <span class="truncate text-faint">via {{ ip.source }}</span>
          </template>
        </div>

        <div class="flex shrink-0 items-center gap-1.5">
          <button
            type="button"
            aria-label="Refresh"
            title="Refresh (Ctrl+R)"
            class="grid h-8 w-8 place-items-center rounded-xl border border-line bg-white text-muted transition-colors duration-100 hover:border-brand-line hover:bg-brand-soft hover:text-brand disabled:opacity-50"
            :disabled="loading"
            @click="check"
          >
            <svg
              viewBox="0 0 24 24"
              class="h-3.5 w-3.5"
              :class="{ 'animate-spin': loading }"
              fill="none"
              stroke="currentColor"
              stroke-width="1.6"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M20.5 12a8.5 8.5 0 1 1-2.49-6.01" />
              <path d="M20.5 4v5h-5" />
            </svg>
          </button>
          <button
            type="button"
            aria-label="Settings"
            title="Settings"
            class="grid h-8 w-8 place-items-center rounded-xl border border-line bg-white text-muted transition-colors duration-100 hover:border-brand-line hover:bg-brand-soft hover:text-brand"
            @click="openSettings"
          >
            <svg
              viewBox="0 0 24 24"
              class="h-3.5 w-3.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="3" />
              <path
                d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"
              />
            </svg>
          </button>
        </div>
      </div>

      <div
        v-if="changeNotice"
        class="mt-3 flex items-start gap-2 rounded-xl border border-brand-line bg-brand-soft px-3 py-2"
      >
        <svg
          viewBox="0 0 24 24"
          class="mt-px h-3.5 w-3.5 shrink-0 text-brand"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M12 8v5" />
          <circle cx="12" cy="16.5" r="0.6" fill="currentColor" stroke="none" />
          <circle cx="12" cy="12" r="9" />
        </svg>
        <div class="min-w-0 flex-1">
          <p class="text-[11px] font-medium text-brand-strong">WAN IP changed</p>
          <p class="mt-0.5 break-all font-mono text-[10.5px] leading-4 text-brand">
            {{ changeNotice.previous ?? "Previous address" }} → {{ changeNotice.current }}
          </p>
        </div>
        <button
          type="button"
          aria-label="Dismiss IP change notice"
          class="grid h-5 w-5 shrink-0 place-items-center rounded-md text-brand/70 transition-colors duration-100 hover:bg-white hover:text-brand"
          @click="dismissChange"
        >
          <svg
            viewBox="0 0 24 24"
            class="h-3 w-3"
            fill="none"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
          >
            <path d="M6 6l12 12M18 6L6 18" />
          </svg>
        </button>
      </div>

      <div class="mt-4 rounded-2xl border border-brand-line bg-[#F3FAF6] px-4 pb-3.5 pt-3">
        <div class="flex items-center justify-between">
          <span class="text-[10px] font-medium uppercase tracking-[0.12em] text-brand"
            >Public address</span
          >
          <span
            v-if="ip"
            class="rounded-[5px] border border-brand-line bg-white px-1.5 py-0.5 font-mono text-[9.5px] font-medium text-brand"
            >IPv{{ ip.version }}</span
          >
        </div>

        <div v-if="loading && !ip" class="space-y-2 py-2">
          <div class="h-7 w-3/5 animate-pulse rounded-md bg-fill"></div>
          <div class="h-3 w-24 animate-pulse rounded bg-shell"></div>
        </div>

        <p
          v-else-if="ip"
          class="mt-1 select-text break-all font-mono font-medium tracking-[-0.02em] tabular-nums text-brand-strong"
          :class="ipSize"
        >
          {{ ip.ip }}
        </p>

        <p v-else class="mt-1 text-[26px] leading-[1.3] font-medium text-[#c2c7cf]">
          Unavailable
        </p>

        <div class="mt-3 flex items-end justify-between gap-3">
          <div class="min-w-0 text-[10.5px] leading-4 text-faint">
            <template v-if="error">
              <span class="text-danger">{{ error }}</span>
            </template>
            <template v-else-if="ip">
              Checked {{ timeLabel(ip.checkedAt) }} · saved locally
            </template>
            <template v-else>Reaching the lookup service…</template>
          </div>

          <button
            type="button"
            class="flex h-8 shrink-0 items-center gap-1.5 rounded-[10px] bg-brand px-3 text-[12px] font-medium text-white transition-colors duration-100 hover:bg-brand-strong disabled:bg-[#c3d6cb]"
            :disabled="!ip"
            @click="copyCurrent"
          >
            <svg
              v-if="copiedMain"
              viewBox="0 0 24 24"
              class="h-3.5 w-3.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.9"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M20 6L9 17l-5-5" />
            </svg>
            <svg
              v-else
              viewBox="0 0 24 24"
              class="h-3.5 w-3.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="9" y="9" width="11" height="11" rx="2" />
              <path d="M5 15V5a2 2 0 0 1 2-2h10" />
            </svg>
            {{ copiedMain ? "Copied" : "Copy" }}
          </button>
        </div>
      </div>

      <div class="mt-4 flex min-h-0 flex-1 flex-col">
        <HistoryList
          :history="visibleHistory"
          :hidden-auto-count="hiddenAutoCount"
          :copied-id="copiedId"
          :confirming-clear="confirmingClear"
          @copy="copyEntry"
          @check="check"
          @clear="askClear"
        />
      </div>

      <div
        class="mt-2.5 flex shrink-0 items-center justify-between border-t border-line pt-2.5 text-[10px] text-faint"
      >
        <span class="inline-flex items-center gap-1">
          <svg viewBox="0 0 24 24" class="h-3 w-3 text-brand" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round">
            <rect x="4.5" y="10.5" width="15" height="10" rx="2.2" />
            <path d="M8 10.5V8a4 4 0 0 1 8 0v2.5" />
          </svg>
          Stored on this device
        </span>
        <span
          class="tabular-nums text-brand"
          :title="
            hiddenAutoCount
              ? `${hiddenAutoCount} automatic check${hiddenAutoCount === 1 ? '' : 's'} hidden by settings`
              : undefined
          "
          >{{ visibleHistory.length }}/{{ HISTORY_LIMIT }} saved</span
        >
      </div>

      <div class="mt-2 shrink-0">
        <button
          type="button"
          class="block max-w-full truncate rounded-[3px] px-0.5 text-left text-[10px] text-faint transition-colors duration-100 hover:text-brand"
          title="Open the repository on GitHub"
          @click="openRepo"
        >
          {{ REPO_LABEL }}
        </button>
      </div>
      </template>

      <SettingsPanel
        v-else
        :settings="settings"
        :saving="savingSettings"
        @close="closeSettings"
        @update="updateSettings"
      />
    </main>
  </div>
</template>
