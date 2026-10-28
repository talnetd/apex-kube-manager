// src/lib/stores/settings.ts
import { writable } from "svelte/store";

export const SETTINGS_STORAGE_KEY = "apex-kube-manager-settings";

export type Density = "compact" | "comfortable";

/** A muted Pulse issue (spec 6.3): hidden until `until`, or until its severity escalates. */
export interface MutedIssue {
  until: number; // epoch ms
  severity?: "critical" | "error" | "warning";
}

export interface Settings {
  notificationsEnabled: boolean;
  density: Density;
  /** Per-context production override: true = prod, false = not prod, absent = auto (spec 4.6). */
  prodOverrides: Record<string, boolean>;
  /** Muted Pulse issues keyed by issue fingerprint. */
  mutedIssues: Record<string, MutedIssue>;
}

export const defaultSettings: Settings = {
  notificationsEnabled: true,
  density: "comfortable",
  prodOverrides: {},
  mutedIssues: {},
};

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

/** Merge a stored settings blob over the defaults, dropping malformed fields. */
export function parseSettings(raw: string | null): Settings {
  if (!raw) return { ...defaultSettings };
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return { ...defaultSettings };
  }
  if (!isRecord(parsed)) return { ...defaultSettings };

  const out: Settings = { ...defaultSettings, prodOverrides: {}, mutedIssues: {} };
  if (typeof parsed.notificationsEnabled === "boolean") {
    out.notificationsEnabled = parsed.notificationsEnabled;
  }
  if (parsed.density === "compact" || parsed.density === "comfortable") {
    out.density = parsed.density;
  }
  if (isRecord(parsed.prodOverrides)) {
    for (const [ctx, v] of Object.entries(parsed.prodOverrides)) {
      if (typeof v === "boolean") out.prodOverrides[ctx] = v;
    }
  }
  if (isRecord(parsed.mutedIssues)) {
    for (const [fp, v] of Object.entries(parsed.mutedIssues)) {
      if (isRecord(v) && typeof v.until === "number") {
        out.mutedIssues[fp] = v as unknown as MutedIssue;
      }
    }
  }
  return out;
}

function getInitialSettings(): Settings {
  if (typeof globalThis.window === "undefined") return { ...defaultSettings };
  return parseSettings(localStorage.getItem(SETTINGS_STORAGE_KEY));
}

export const settings = writable<Settings>(getInitialSettings());

// Persist settings changes to localStorage. Identical writes are skipped so a
// value received through the `storage` event doesn't echo back.
settings.subscribe((value: Settings) => {
  if (typeof globalThis.window !== "undefined") {
    const next = JSON.stringify(value);
    if (localStorage.getItem(SETTINGS_STORAGE_KEY) !== next) {
      localStorage.setItem(SETTINGS_STORAGE_KEY, next);
    }
  }
});

// Detail and terminal windows are separate webviews sharing localStorage:
// follow changes made in another window live (density, production marks).
if (typeof globalThis.window !== "undefined") {
  globalThis.window.addEventListener("storage", (e: StorageEvent) => {
    if (e.key === SETTINGS_STORAGE_KEY) settings.set(parseSettings(e.newValue));
  });
}

// Helper to toggle notifications
export function toggleNotifications() {
  settings.update((s) => ({ ...s, notificationsEnabled: !s.notificationsEnabled }));
}

// Helper to set notifications enabled/disabled
export function setNotificationsEnabled(enabled: boolean) {
  settings.update((s) => ({ ...s, notificationsEnabled: enabled }));
}

const MUTE_MS = 60 * 60 * 1000;

/** Mute a Pulse issue for 1h (spec 6.3). */
export function muteIssue(fingerprint: string, severity?: MutedIssue["severity"], now = Date.now()) {
  settings.update((s) => ({
    ...s,
    mutedIssues: { ...s.mutedIssues, [fingerprint]: { until: now + MUTE_MS, severity } },
  }));
}

export function unmuteIssue(fingerprint: string) {
  settings.update((s) => {
    if (!(fingerprint in s.mutedIssues)) return s;
    const rest = { ...s.mutedIssues };
    delete rest[fingerprint];
    return { ...s, mutedIssues: rest };
  });
}
