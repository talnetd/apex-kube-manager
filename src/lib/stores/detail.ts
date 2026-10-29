// src/lib/stores/detail.ts
// Shared-context awareness for one detail window (spec 13.3): follows the
// app's `context-changed` broadcast and derives read-only.
import { derived, get, writable } from 'svelte/store';
import { toast } from './toast';
import { acceptContextEvent, type ContextChangedEvent } from './kubernetes';
import { blockedMessage, canRun, detailAccess, type DetailAction } from '../utils/detail-access';

export const lockedContext = writable<string>('');
export const sharedContext = writable<string | null>(null);

export const access = derived([lockedContext, sharedContext], ([$l, $s]) => detailAccess($l, $s));
export const detailReadOnly = derived(access, ($a) => $a.readOnly);

let started = false;
let lastSeq = 0;

export async function startDetailContextSync(locked: string) {
  lockedContext.set(locked);
  if (started || typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return;
  started = true;
  const { invoke } = await import('@tauri-apps/api/core');
  const { listen } = await import('@tauri-apps/api/event');
  await listen<ContextChangedEvent>('context-changed', (e) => {
    if (!acceptContextEvent(lastSeq, e.payload)) return;
    lastSeq = e.payload.seq;
    sharedContext.set(e.payload.context);
  });
  try {
    // Seq-ordered like the events: if one already arrived, it's newer.
    const cur = await invoke<ContextChangedEvent | null>('get_shared_context');
    if (cur && acceptContextEvent(lastSeq, cur, true)) {
      lastSeq = cur.seq;
      sharedContext.set(cur.context);
    } else if (!cur && lastSeq === 0) {
      sharedContext.set(await invoke<string>('get_current_context'));
    }
  } catch {
    /* unknown: stay writable rather than lock the user out */
  }
}

/** Call first in every mutating handler: false (and a toast) while read-only. */
export function guard(action: DetailAction): boolean {
  const a = get(access);
  if (canRun(a, action)) return true;
  toast(blockedMessage(a, action), { health: 'warning' });
  return false;
}

/** The banner's button: switch the shared context back to this window's. */
export async function switchBack() {
  const { invoke } = await import('@tauri-apps/api/core');
  await invoke('switch_context', { contextName: get(lockedContext) });
}
