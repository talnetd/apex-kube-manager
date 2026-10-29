// src/lib/components/ui/data-table.ts
// Types and pure helpers for DataTable.svelte (spec 5.4).
import type { Snippet } from 'svelte';
import type { Health, HealthResult } from '../../utils/health';
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
  if (Array.isArray(v)) return v.length ? v.join(', ') : '–';
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

/**
 * Window-width breakpoints (spec 13.2), for `Column.hideBelow`:
 * sm < 640 (side panel), md 640-1199 (half screen), lg >= 1200 (full).
 * `hideBelow: BP.md` keeps a column only from md up; `BP.wide` also drops it
 * in the narrow half of md, where the icon sidebar still takes 52px.
 */
export const BP = { md: 640, wide: 900, lg: 1200 } as const;

/** Map a store's load status onto the table's states. */
export function tableStateOf(status: string | undefined): TableState {
  switch (status) {
    case 'error':
      return 'error';
    case 'forbidden':
      return 'forbidden';
    case 'ready':
      return 'ready';
    default:
      return 'loading';
  }
}

export type FilterField<T> = keyof T | ((r: T) => unknown);

function fieldText<T>(r: T, f: FilterField<T>): string {
  const v = typeof f === 'function' ? f(r) : (r as Record<string, unknown>)[f as string];
  if (v == null) return '';
  return Array.isArray(v) ? v.join(' ') : String(v);
}

const QUANTITY_SUFFIX: Record<string, number> = {
  '': 1,
  m: 1e-3,
  k: 1e3,
  M: 1e6,
  G: 1e9,
  T: 1e12,
  P: 1e15,
  Ki: 2 ** 10,
  Mi: 2 ** 20,
  Gi: 2 ** 30,
  Ti: 2 ** 40,
  Pi: 2 ** 50,
};

/**
 * A Kubernetes quantity ("500Mi", "10Gi", "1.5T") as a number, so capacity
 * columns sort by size rather than by text. Null when it doesn't parse.
 */
export function quantityValue(q: string | null | undefined): number | null {
  const m = q?.trim().match(/^(\d+(?:\.\d+)?)([a-zA-Z]*)$/);
  if (!m) return null;
  const mult = QUANTITY_SUFFIX[m[2]];
  return mult === undefined ? null : parseFloat(m[1]) * mult;
}

/** Segment order in PageHeader: worst first, so keys 1-5 hit the problems. */
export const SEGMENT_ORDER: readonly Health[] = ['failed', 'warning', 'progressing', 'unknown', 'healthy', 'neutral'];

/** Rows per health state, for the PageHeader segment counts. */
export function healthCounts<T>(rows: readonly T[], health: (r: T) => HealthResult): Record<Health, number> {
  const counts: Record<Health, number> = { failed: 0, warning: 0, progressing: 0, unknown: 0, healthy: 0, neutral: 0 };
  for (const r of rows) counts[health(r).health]++;
  return counts;
}

/** Case-insensitive substring filter across the given fields (arrays joined). */
export function filterRows<T>(rows: readonly T[], query: string, fields: readonly FilterField<T>[]): T[] {
  const q = query.trim().toLowerCase();
  if (!q) return [...rows];
  return rows.filter((r) => fields.some((f) => fieldText(r, f).toLowerCase().includes(q)));
}
