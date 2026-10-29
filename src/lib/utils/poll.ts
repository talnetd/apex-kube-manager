// src/lib/utils/poll.ts
/** Default list refresh interval (CLAUDE.md, Architecture Note 3). */
export const LIST_POLL_MS = 10000;

/**
 * Load now, then every `ms`. Returns the cleanup, so it can be returned
 * straight from a `$effect` that re-runs on context, namespace or refresh:
 *
 *   $effect(() => {
 *     if (!$currentContext) return;
 *     void $refreshTrigger;
 *     const ns = $selectedNamespace;
 *     return poll(() => loadConfigMaps(ns));
 *   });
 */
export function poll(load: () => unknown, ms = LIST_POLL_MS): () => void {
  load();
  const id = setInterval(load, ms);
  return () => clearInterval(id);
}
