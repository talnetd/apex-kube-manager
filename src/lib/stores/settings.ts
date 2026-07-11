// src/lib/stores/settings.ts
import { writable } from "svelte/store";

const SETTINGS_STORAGE_KEY = "apex-kube-manager-settings";

interface Settings {
  notificationsEnabled: boolean;
}

const defaultSettings: Settings = {
  notificationsEnabled: true,
};

function getInitialSettings(): Settings {
  if (typeof globalThis.window === "undefined") return defaultSettings;

  const stored = localStorage.getItem(SETTINGS_STORAGE_KEY);
  if (stored) {
    try {
      return { ...defaultSettings, ...JSON.parse(stored) };
    } catch {
      return defaultSettings;
    }
  }
  return defaultSettings;
}

export const settings = writable<Settings>(getInitialSettings());

// Persist settings changes to localStorage
settings.subscribe((value: Settings) => {
  if (typeof globalThis.window !== "undefined") {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(value));
  }
});

// Helper to toggle notifications
export function toggleNotifications() {
  settings.update((s) => ({ ...s, notificationsEnabled: !s.notificationsEnabled }));
}

// Helper to set notifications enabled/disabled
export function setNotificationsEnabled(enabled: boolean) {
  settings.update((s) => ({ ...s, notificationsEnabled: enabled }));
}
