<script setup lang="ts">
import { INTERVAL_OPTIONS, type Settings } from "../lib/api";

defineProps<{
  settings: Settings;
  saving: boolean;
}>();

const emit = defineEmits<{
  close: [];
  update: [patch: Partial<Settings>];
}>();

function minutesLabel(value: number): string {
  return value === 60 ? "60 minutes (1 hour)" : `${value} minutes`;
}

function onIntervalChange(event: Event): void {
  const value = Number((event.target as HTMLSelectElement).value);
  emit("update", { intervalMinutes: value });
}
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col">
    <div class="flex items-center justify-between gap-3">
      <div class="flex items-center gap-2">
        <span class="text-[10.5px] font-medium uppercase tracking-[0.12em] text-faint"
          >Settings</span
        >
        <span
          class="rounded-full bg-brand-soft px-1.5 py-px text-[10px] font-medium text-brand"
          >Preferences</span
        >
      </div>
      <button
        type="button"
        aria-label="Close settings"
        title="Close settings"
        class="grid h-7 w-7 place-items-center rounded-lg text-faint transition-colors duration-100 hover:bg-brand-soft hover:text-brand"
        @click="emit('close')"
      >
        <svg
          viewBox="0 0 24 24"
          class="h-3.5 w-3.5"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
        >
          <path d="M6 6l12 12M18 6L6 18" />
        </svg>
      </button>
    </div>

    <p class="mt-2 text-[11px] leading-4 text-faint">
      Automatic checks run quietly while the app is open.
    </p>

    <div class="mt-3 space-y-2">
      <div class="rounded-2xl border border-line px-3.5 py-3">
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="text-[12.5px] font-medium text-ink">Check automatically</p>
            <p class="mt-0.5 text-[10.5px] leading-4 text-faint">
              Look up your WAN IP on a schedule.
            </p>
          </div>
          <button
            type="button"
            role="switch"
            aria-label="Check automatically"
            :aria-checked="settings.autoCheck"
            class="relative h-[22px] w-[38px] shrink-0 rounded-full border transition-colors duration-150"
            :class="
              settings.autoCheck
                ? 'border-brand bg-brand'
                : 'border-[#d5ddd7] bg-[#e8eeea]'
            "
            @click="emit('update', { autoCheck: !settings.autoCheck })"
          >
            <span
              class="absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full bg-white shadow-sm transition-all duration-150"
              :class="settings.autoCheck ? 'left-[19px]' : 'left-[2px]'"
            />
          </button>
        </div>

        <div
          class="mt-2.5 flex items-center justify-between gap-3 border-t border-line pt-2.5 transition-opacity duration-150"
          :class="{ 'opacity-45': !settings.autoCheck }"
        >
          <label for="auto-interval" class="text-[11.5px] text-muted">Every</label>
          <select
            id="auto-interval"
            :value="settings.intervalMinutes"
            :disabled="!settings.autoCheck"
            class="h-7 rounded-lg border border-line bg-white px-2 pr-6 text-[11.5px] text-ink transition-colors duration-100 disabled:bg-fill disabled:text-faint"
            @change="onIntervalChange"
          >
            <option v-for="minutes in INTERVAL_OPTIONS" :key="minutes" :value="minutes">
              {{ minutesLabel(minutes) }}
            </option>
          </select>
        </div>
      </div>

      <div class="flex items-start justify-between gap-3 rounded-2xl border border-line px-3.5 py-3">
        <div class="min-w-0">
          <p class="text-[12.5px] font-medium text-ink">Notify when the IP changes</p>
          <p class="mt-0.5 text-[10.5px] leading-4 text-faint">
            Windows notification and an in-app banner.
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-label="Notify when the IP changes"
          :aria-checked="settings.notifyOnChange"
          class="relative h-[22px] w-[38px] shrink-0 rounded-full border transition-colors duration-150"
          :class="
            settings.notifyOnChange
              ? 'border-brand bg-brand'
              : 'border-[#d5ddd7] bg-[#e8eeea]'
          "
          @click="emit('update', { notifyOnChange: !settings.notifyOnChange })"
        >
          <span
            class="absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full bg-white shadow-sm transition-all duration-150"
            :class="settings.notifyOnChange ? 'left-[19px]' : 'left-[2px]'"
          />
        </button>
      </div>

      <div class="flex items-start justify-between gap-3 rounded-2xl border border-line px-3.5 py-3">
        <div class="min-w-0">
          <p class="text-[12.5px] font-medium text-ink">Minimize to system tray</p>
          <p class="mt-0.5 text-[10.5px] leading-4 text-faint">
            Keep running in the background. Click the tray icon to reopen.
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-label="Minimize to system tray"
          :aria-checked="settings.minimizeToTray"
          class="relative h-[22px] w-[38px] shrink-0 rounded-full border transition-colors duration-150"
          :class="
            settings.minimizeToTray
              ? 'border-brand bg-brand'
              : 'border-[#d5ddd7] bg-[#e8eeea]'
          "
          @click="emit('update', { minimizeToTray: !settings.minimizeToTray })"
        >
          <span
            class="absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full bg-white shadow-sm transition-all duration-150"
            :class="settings.minimizeToTray ? 'left-[19px]' : 'left-[2px]'"
          />
        </button>
      </div>

      <div class="flex items-start justify-between gap-3 rounded-2xl border border-line px-3.5 py-3">
        <div class="min-w-0">
          <p class="text-[12.5px] font-medium text-ink">Show auto-checks in History</p>
          <p class="mt-0.5 text-[10.5px] leading-4 text-faint">
            List periodically checked IPs in the history section.
          </p>
        </div>
        <button
          type="button"
          role="switch"
          aria-label="Show auto-checks in History"
          :aria-checked="settings.showAutoChecks"
          class="relative h-[22px] w-[38px] shrink-0 rounded-full border transition-colors duration-150"
          :class="
            settings.showAutoChecks
              ? 'border-brand bg-brand'
              : 'border-[#d5ddd7] bg-[#e8eeea]'
          "
          @click="emit('update', { showAutoChecks: !settings.showAutoChecks })"
        >
          <span
            class="absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full bg-white shadow-sm transition-all duration-150"
            :class="settings.showAutoChecks ? 'left-[19px]' : 'left-[2px]'"
          />
        </button>
      </div>
    </div>

    <div class="mt-auto flex items-center justify-between border-t border-line pt-2.5">
      <span class="inline-flex items-center gap-1 text-[10px] text-faint">
        <svg
          viewBox="0 0 24 24"
          class="h-3 w-3 text-brand"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        >
          <rect x="4.5" y="10.5" width="15" height="10" rx="2.2" />
          <path d="M8 10.5V8a4 4 0 0 1 8 0v2.5" />
        </svg>
        {{ saving ? "Saving…" : "Saved on this device" }}
      </span>
      <button
        type="button"
        class="h-7 rounded-lg bg-brand px-3 text-[11.5px] font-medium text-white transition-colors duration-100 hover:bg-brand-strong"
        @click="emit('close')"
      >
        Done
      </button>
    </div>
  </section>
</template>
