// src/lib/utils/selection.ts
// Keyboard selection anchoring (spec 5.4). Rows re-order under a positional
// selection (watch updates, sortable live columns), so the selection follows
// the row's key instead of its index. Without this, Enter opens a row the
// user never picked. Moved here from NodeList so every DataTable gets it.

/**
 * Where the selection should be after `keys` (the new row order) arrives.
 *
 * - No anchor: keep the index, clamped (-1 stays -1).
 * - Anchored row still present: its new index.
 * - Anchored row gone: the row now at the same index (clamped), so the
 *   cursor doesn't jump to the top. Empty table -> -1.
 */
export function anchorSelection(
  anchoredKey: string | null,
  previousIndex: number,
  keys: readonly string[]
): number {
  if (keys.length === 0) return -1;
  if (anchoredKey !== null) {
    const idx = keys.indexOf(anchoredKey);
    if (idx !== -1) return idx;
  }
  if (previousIndex < 0) return -1;
  return Math.min(previousIndex, keys.length - 1);
}
