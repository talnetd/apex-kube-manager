// src/lib/components/ui/data-table.ts
// Types and pure helpers for DataTable.svelte (spec 5.4).
import type { Snippet } from 'svelte';
import type { HealthResult } from '../../utils/health';
import { parseAge } from '../../utils/sort';

export type SortDir = 'asc' | 'desc';
export interface TableSort {
  id: string;
  dir: SortDir;
}

export interface Column<T> {
  id: string;
  label: string;
  /** Omit = not sortable. */
  sortKey?: keyof T | ((r: T) => string | number | null | undefined);
  align?: 'left' | 'right';
  /** e.g. '28%', '96px'. The name column is usually left flexible. */
  width?: string;
  mono?: boolean;
  /** tabular-nums + right-aligned. */
  numeric?: boolean;
  /** Hide the column when the viewport is narrower than this (px). */
  hideBelow?: number;
  /** The row's name cell: text-primary 500, truncates with a title tooltip. */
  primary?: boolean;
  /** Custom render; default is String(row[id]). */
  cell?: Snippet<[T]>;
  /** Tooltip text for truncated cells; defaults to the cell text. */
  title?: (r: T) => string | undefined;
}

/** Synthetic column id: sorts by health rank, then name (problems first). */
export const HEALTH_SORT_ID = 'health';

export type TableState = 'loading' | 'ready' | 'error' | 'forbidden';

export function cellText<T>(row: T, col: Column<T>): string {
  const v = (row as Record<string, unknown>)[col.id];
  return v == null || v === '' ? '–' : String(v);
}

function sortValue<T>(row: T, col: Column<T>): string | number | null | undefined {
  if (typeof col.sortKey === 'function') return col.sortKey(row);
  if (col.sortKey !== undefined) return (row as Record<string, unknown>)[col.sortKey as string] as never;
  return undefined;
}

function compareValues(a: unknown, b: unknown, isAge: boolean): number {
  const aNull = a == null || a === '';
  const bNull = b == null || b === '';
  if (aNull && bNull) return 0;
  if (aNull) return 1; // empties sink, regardless of direction (applied below)
  if (bNull) return -1;
  if (typeof a === 'number' && typeof b === 'number') return a - b;
  if (isAge) return parseAge(String(a)) - parseAge(String(b));
  return String(a).localeCompare(String(b), undefined, { numeric: true, sensitivity: 'base' });
}

/**
 * Sort rows for display. Ties break on the row key so the order is stable
 * across updates (which keeps anchored selection steady).
 */
export function sortRows<T>(
  rows: readonly T[],
  columns: readonly Column<T>[],
  sort: TableSort | null,
  rowKey: (r: T) => string,
  health?: (r: T) => HealthResult
): T[] {
  const out = [...rows];
  if (!sort) {
    return out;
  }
  const dir = sort.dir === 'asc' ? 1 : -1;
  const nameCol = columns.find((c) => c.primary) ?? columns.find((c) => c.id === 'name');
  const nameOf = (r: T) => (nameCol ? String(sortValue(r, nameCol) ?? cellText(r, nameCol)) : rowKey(r));

  if (sort.id === HEALTH_SORT_ID) {
    return out.sort((a, b) => {
      const ra = health ? health(a).rank : 0;
      const rb = health ? health(b).rank : 0;
      if (ra !== rb) return (ra - rb) * dir;
      const n = nameOf(a).localeCompare(nameOf(b), undefined, { numeric: true });
      return n !== 0 ? n : rowKey(a).localeCompare(rowKey(b));
    });
  }

  const col = columns.find((c) => c.id === sort.id);
  if (!col || col.sortKey === undefined) return out;
  const isAge = col.id === 'age' || col.id.endsWith('_age');
  return out.sort((a, b) => {
    const va = sortValue(a, col);
    const vb = sortValue(b, col);
    const aNull = va == null || va === '';
    const bNull = vb == null || vb === '';
    // Empty values always sink to the bottom.
    if (aNull !== bNull) return aNull ? 1 : -1;
    const c = compareValues(va, vb, isAge) * dir;
    return c !== 0 ? c : rowKey(a).localeCompare(rowKey(b));
  });
}

/** Header click cycle: asc -> desc -> back to the default sort. */
export function nextSort(current: TableSort | null, columnId: string, fallback: TableSort | null): TableSort | null {
  if (!current || current.id !== columnId) return { id: columnId, dir: 'asc' };
  if (current.dir === 'asc') return { id: columnId, dir: 'desc' };
  if (fallback && fallback.id === columnId) return { id: columnId, dir: 'asc' };
  return fallback;
}
