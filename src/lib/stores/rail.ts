// src/lib/stores/rail.ts
import { writable } from "svelte/store";

// Whether the Pulse rail (capacity + live events) is open.
export const pulseRailOpen = writable(false);
