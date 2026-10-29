// src/lib/stores/windows.ts
// Number of main Apex windows sharing the context (spec 13.3). The header's
// linked-windows marker reads it and hides itself at 1. Multi-window (⌘N,
// the `windows://count` event, spec 13.6) isn't built yet, so this stays 1
// until PR6b wires the event in.
import { writable } from "svelte/store";

export const mainWindowCount = writable<number>(1);
