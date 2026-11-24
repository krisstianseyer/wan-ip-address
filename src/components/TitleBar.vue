<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "../lib/api";

const win = isTauri ? getCurrentWindow() : null;

const minimize = () => win?.minimize();
const toggleMaximize = () => win?.toggleMaximize();
const close = () => win?.close();
</script>

<template>
  <header
    data-tauri-drag-region
    class="flex h-9 shrink-0 select-none items-center justify-between pl-3 pr-1.5"
    @dblclick="toggleMaximize"
  >
    <div data-tauri-drag-region class="flex min-w-0 items-center gap-2">
      <span
        class="grid h-4.5 w-4.5 place-items-center rounded-[5px] bg-primary text-[9px] font-semibold tracking-tight text-white"
        >IP</span
      >
      <span class="truncate text-[12.5px] font-medium tracking-[-0.01em] text-ink/80"
        >WAN IP Address</span
      >
      <span
        class="rounded-[4px] border border-line/80 bg-white/60 px-1 py-px text-[9.5px] font-medium uppercase tracking-[0.08em] text-faint"
        >v1.0.0</span
      >
    </div>

    <div class="flex items-center">
      <button
        type="button"
        aria-label="Minimize"
        class="grid h-8 w-9 place-items-center rounded-md text-muted transition-colors duration-100 hover:bg-fill hover:text-ink"
        @click="minimize"
      >
        <svg viewBox="0 0 24 24" class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <path d="M5 12h14" />
        </svg>
      </button>
      <button
        type="button"
        aria-label="Maximize"
        class="grid h-8 w-9 place-items-center rounded-md text-muted transition-colors duration-100 hover:bg-fill hover:text-ink"
        @click="toggleMaximize"
      >
        <svg viewBox="0 0 24 24" class="h-3 w-3" fill="none" stroke="currentColor" stroke-width="1.6">
          <rect x="5.5" y="5.5" width="13" height="13" rx="1.8" />
        </svg>
      </button>
      <button
        type="button"
        aria-label="Close"
        class="grid h-8 w-9 place-items-center rounded-md text-muted transition-colors duration-100 hover:bg-[#e5484d] hover:text-white"
        @click="close"
      >
        <svg viewBox="0 0 24 24" class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round">
          <path d="M6 6l12 12M18 6L6 18" />
        </svg>
      </button>
    </div>
  </header>
</template>
