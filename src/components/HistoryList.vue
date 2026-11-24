<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { relativeTime, type HistoryEntry } from "../lib/api";

const props = defineProps<{
  history: HistoryEntry[];
  copiedId: string | null;
  confirmingClear: boolean;
}>();

const emit = defineEmits<{
  copy: [entry: HistoryEntry];
  clear: [];
  check: [];
}>();

const filter = ref<"all" | "4" | "6">("all");
const now = ref(Date.now());
let tick: number | undefined;

onMounted(() => {
  tick = window.setInterval(() => (now.value = Date.now()), 30_000);
});

onUnmounted(() => window.clearInterval(tick));

function timeAgo(iso: string): string {
  void now.value;
  return relativeTime(iso);
}

const items = computed(() => {
  if (filter.value === "all") return props.history;
  return props.history.filter((entry) =>
    filter.value === "4" ? entry.version === 4 : entry.version === 6,
  );
});

const tabs = [
  { key: "all", label: "All" },
  { key: "4", label: "IPv4" },
  { key: "6", label: "IPv6" },
] as const;
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col">
    <div class="mb-2 flex items-center justify-between">
      <div class="flex items-center gap-2.5">
        <span class="text-[10.5px] font-medium uppercase tracking-[0.12em] text-faint"
          >History</span
        >
        <span
          v-if="history.length"
          class="rounded-full bg-brand-soft px-1.5 py-px text-[10px] font-medium tabular-nums text-brand"
          >{{ history.length }}</span
        >
      </div>
      <button
        v-if="history.length"
        type="button"
        :aria-label="confirmingClear ? 'Confirm clear history' : 'Clear history'"
        :title="confirmingClear ? 'Click again to clear' : 'Clear history'"
        class="flex h-6 items-center justify-center rounded-md text-[10.5px] font-medium transition-colors duration-100"
        :class="
          confirmingClear
            ? 'bg-[#FDF1F0] px-2 text-danger'
            : 'w-6 text-faint hover:bg-fill hover:text-ink'
        "
        @click="emit('clear')"
      >
        <span v-if="confirmingClear">Clear?</span>
        <svg v-else viewBox="0 0 24 24" class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 7h16M9 7V5.2A1.2 1.2 0 0 1 10.2 4h3.6A1.2 1.2 0 0 1 15 5.2V7" />
          <path d="M6.4 7l.8 12.1A1.9 1.9 0 0 0 9.1 21h5.8a1.9 1.9 0 0 0 1.9-1.9L17.6 7" />
        </svg>
      </button>
    </div>

    <div class="mb-2.5 flex items-center gap-1">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        type="button"
        class="rounded-md px-2 py-1 text-[11.5px] font-medium transition-colors duration-100"
        :class="
          filter === tab.key
            ? 'bg-brand text-white'
            : 'text-faint hover:bg-brand-soft hover:text-brand'
        "
        @click="filter = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <div
      v-if="items.length"
      class="scroll-fade -mx-1 min-h-0 flex-1 overflow-y-auto px-1 pb-2"
    >
      <button
        v-for="entry in items"
        :key="entry.id"
        type="button"
        class="group mb-0.5 flex w-full items-center gap-2 rounded-lg px-2 py-2 text-left transition-colors duration-100 hover:bg-brand-soft/70"
        :title="`Copy ${entry.ip}`"
        @click="emit('copy', entry)"
      >
        <span
          class="w-6 shrink-0 rounded-[4px] border border-brand-line bg-brand-soft/60 py-px text-center font-mono text-[9.5px] font-medium text-brand"
          >v{{ entry.version }}</span
        >
        <span class="min-w-0 flex-1 truncate font-mono text-[12.5px] tracking-[-0.01em] text-ink/90">{{
          entry.ip
        }}</span>
        <span
          v-if="copiedId === entry.id"
          class="shrink-0 text-[10.5px] font-medium text-positive"
          >Copied</span
        >
        <span
          v-else
          class="shrink-0 text-[10.5px] tabular-nums text-faint group-hover:text-muted"
          >{{ timeAgo(entry.checkedAt) }}</span
        >
      </button>
    </div>

    <div
      v-else
      class="flex min-h-0 flex-1 flex-col items-center justify-center pb-6 text-center"
    >
      <div class="mb-2.5 grid h-11 w-11 place-items-center rounded-full bg-brand-soft">
        <svg viewBox="0 0 24 24" class="h-[22px] w-[22px] text-brand" fill="none" stroke="currentColor" stroke-width="1.25" stroke-linecap="round">
          <circle cx="12" cy="12" r="9" />
          <path d="M12 7.2V12l3.1 2" />
        </svg>
      </div>
      <p class="text-[12.5px] font-medium text-muted">
        {{ filter === "all" ? "No history yet" : `No IPv${filter} addresses` }}
      </p>
      <p class="mt-0.5 text-[11.5px] text-faint">
        {{
          filter === "all"
            ? "Checks are saved automatically."
            : "Try another filter or check again."
        }}
      </p>
      <button
        type="button"
        class="mt-3 rounded-xl border border-brand-line bg-white px-3 py-1.5 text-[12px] font-medium text-brand transition-colors duration-100 hover:bg-brand hover:text-white"
        @click="emit('check')"
      >
        Check now
      </button>
    </div>
  </section>
</template>

<style scoped>
.scroll-fade {
  mask-image: linear-gradient(to bottom, #000 calc(100% - 20px), transparent 100%);
}
</style>
