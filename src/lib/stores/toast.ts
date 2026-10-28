// src/lib/stores/toast.ts
// Toasts (spec 5.7): bottom-right above the status bar, 4s, max 3 stacked.
import { writable } from "svelte/store";
import type { Health } from "../utils/health";

export interface Toast {
  id: number;
  message: string;
  health?: Health;
  /** Custom leading icon instead of the health icon (e.g. the prod shield in PR6). */
  icon?: "shield";
  duration: number;
}

export interface ToastOptions {
  health?: Health;
  icon?: Toast["icon"];
  duration?: number;
}

export const MAX_TOASTS = 3;
export const DEFAULT_TOAST_MS = 4000;

export const toasts = writable<Toast[]>([]);

let nextId = 1;
const timers = new Map<number, ReturnType<typeof setTimeout>>();

export function dismissToast(id: number) {
  const t = timers.get(id);
  if (t) clearTimeout(t);
  timers.delete(id);
  toasts.update((list) => list.filter((x) => x.id !== id));
}

export function toast(message: string, opts: ToastOptions = {}): number {
  const id = nextId++;
  const duration = opts.duration ?? DEFAULT_TOAST_MS;
  const item: Toast = { id, message, health: opts.health, icon: opts.icon, duration };
  toasts.update((list) => {
    const next = [...list, item];
    // Oldest drop off first; clear their timers too.
    while (next.length > MAX_TOASTS) {
      const dropped = next.shift()!;
      const t = timers.get(dropped.id);
      if (t) clearTimeout(t);
      timers.delete(dropped.id);
    }
    return next;
  });
  if (duration > 0) {
    timers.set(id, setTimeout(() => dismissToast(id), duration));
  }
  return id;
}
