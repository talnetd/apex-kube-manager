// src/lib/stores/windows.ts
// Main Apex windows sharing the context (spec 13.3, 13.4). The header's
// linked-windows marker reads the count and hides itself at 1. Rust emits
// `windows-changed` when a main window opens or closes.
import { writable } from "svelte/store";

export const mainWindowCount = writable<number>(1);

let started = false;

/** Start tracking the count (main windows). Idempotent. */
export async function trackWindowCount() {
  if (started || typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return;
  started = true;
  const { invoke } = await import("@tauri-apps/api/core");
  const { listen } = await import("@tauri-apps/api/event");
  await listen<number>("windows-changed", (e) => mainWindowCount.set(Math.max(1, e.payload)));
  try {
    mainWindowCount.set(Math.max(1, await invoke<number>("get_main_window_count")));
  } catch {
    /* older backend: stays 1 */
  }
}

/** ⌘N / Ctrl+N and File > New Window. */
export async function openNewWindow() {
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("open_main_window");
}
