// src/lib/stores/density.ts
// Density (spec 3.1): `data-density` on <html> drives CSS variables in
// app.css. Components never branch on density in JS; they read the variables.
import { derived, get, type Readable } from "svelte/store";
import { settings, type Density } from "./settings";

export type { Density };

export const density: Readable<Density> = derived(settings, ($s) => $s.density);

export function setDensity(value: Density) {
  settings.update((s) => (s.density === value ? s : { ...s, density: value }));
}

/** Flip between compact and comfortable; returns the new value. */
export function toggleDensity(): Density {
  const next: Density = get(density) === "compact" ? "comfortable" : "compact";
  setDensity(next);
  return next;
}

export function densityLabel(value: Density): string {
  return value === "compact" ? "Compact" : "Comfortable";
}

// Apply to the document. Importing this module once per window is enough
// (App.svelte, DetailWindow.svelte, TerminalWindow.svelte).
density.subscribe((value) => {
  if (typeof document !== "undefined") {
    document.documentElement.dataset.density = value;
  }
});
