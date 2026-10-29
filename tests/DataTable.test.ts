import { describe, it, expect, vi, afterEach } from 'vitest';
import { mount, unmount, flushSync } from 'svelte';
import { writable, get } from 'svelte/store';
import TableHarness from './fixtures/TableHarness.svelte';
import { selectedRowIndex, totalRows, keyboardNavActive } from '../src/lib/stores/keyboard';
import { anchorSelection } from '../src/lib/utils/selection';
import { sortRows, nextSort, type Column } from '../src/lib/components/ui/data-table';

interface Row {
  name: string;
  cpu: number;
  rank?: number;
}

describe('anchorSelection', () => {
  it('follows the anchored key to its new index', () => {
    expect(anchorSelection('b', 1, ['c', 'a', 'b'])).toBe(2);
  });
  it('keeps the index (clamped) when the anchored row disappears', () => {
    expect(anchorSelection('b', 1, ['a', 'c'])).toBe(1);
    expect(anchorSelection('c', 2, ['a'])).toBe(0);
  });
  it('returns -1 for an empty table or no selection', () => {
    expect(anchorSelection('a', 0, [])).toBe(-1);
    expect(anchorSelection(null, -1, ['a'])).toBe(-1);
  });
});

describe('sortRows / nextSort', () => {
  const cols: Column<Row>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'cpu', label: 'CPU', sortKey: 'cpu', numeric: true },
  ];
  const rows: Row[] = [
    { name: 'web-2', cpu: 5 },
    { name: 'api-1', cpu: 50, rank: 0 },
    { name: 'web-10', cpu: 20 },
  ];
  const key = (r: Row) => r.name;
  const health = (r: Row) => ({ health: 'healthy' as const, label: '', rank: r.rank ?? 4 });

  it('sorts numerically and naturally', () => {
    expect(sortRows(rows, cols, { id: 'cpu', dir: 'desc' }, key).map(key)).toEqual(['api-1', 'web-10', 'web-2']);
    expect(sortRows(rows, cols, { id: 'name', dir: 'asc' }, key).map(key)).toEqual(['api-1', 'web-2', 'web-10']);
  });

  it('health sort puts problems first, then name', () => {
    expect(sortRows(rows, cols, { id: 'health', dir: 'asc' }, key, health).map(key)).toEqual([
      'api-1',
      'web-2',
      'web-10',
    ]);
  });

  it('header click cycles asc -> desc -> default', () => {
    const def = { id: 'health', dir: 'asc' as const };
    let s = nextSort(def, 'cpu', def);
    expect(s).toEqual({ id: 'cpu', dir: 'asc' });
    s = nextSort(s, 'cpu', def);
    expect(s).toEqual({ id: 'cpu', dir: 'desc' });
    s = nextSort(s, 'cpu', def);
    expect(s).toEqual(def);
  });
});

describe('DataTable selection anchoring', () => {
  let app: ReturnType<typeof mount> | null = null;
  afterEach(() => {
    if (app) unmount(app);
    app = null;
    document.body.innerHTML = '';
  });

  it('keeps the selection on the same row key across re-sorts and watch updates', () => {
    const rows = writable<Row[]>([
      { name: 'a', cpu: 10 },
      { name: 'b', cpu: 20 },
      { name: 'c', cpu: 30 },
    ]);
    const onOpen = vi.fn();
    // Sorted by CPU descending: c, b, a. The CPU column changes every poll.
    app = mount(TableHarness, {
      target: document.body,
      props: { rows, onOpen, defaultSort: { id: 'cpu', dir: 'desc' } },
    });
    flushSync();
    expect(get(totalRows)).toBe(3);

    keyboardNavActive.set(true);
    selectedRowIndex.set(1); // "b"
    flushSync();
    expect(document.querySelector('tr[aria-current="true"]')?.getAttribute('data-key')).toBe('b');

    // A poll reorders the rows: b is now the busiest, so it sorts first.
    rows.set([
      { name: 'a', cpu: 10 },
      { name: 'b', cpu: 90 },
      { name: 'c', cpu: 30 },
    ]);
    flushSync();
    expect(get(selectedRowIndex)).toBe(0);
    expect(document.querySelector('tr[aria-current="true"]')?.getAttribute('data-key')).toBe('b');

    // Enter opens the anchored row, not whatever inherited the old index.
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
    expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ name: 'b' }));

    // Another reorder, plus a new row.
    rows.set([
      { name: 'a', cpu: 95 },
      { name: 'b', cpu: 40 },
      { name: 'c', cpu: 30 },
      { name: 'd', cpu: 99 },
    ]);
    flushSync();
    expect(get(totalRows)).toBe(4);
    expect(document.querySelector('tr[aria-current="true"]')?.getAttribute('data-key')).toBe('b');
    expect(get(selectedRowIndex)).toBe(2);

    // The selected row is deleted: selection stays at the same index, no jump to top.
    rows.set([
      { name: 'a', cpu: 95 },
      { name: 'c', cpu: 30 },
      { name: 'd', cpu: 99 },
    ]);
    flushSync();
    expect(get(selectedRowIndex)).toBe(2);
    expect(document.querySelector('tr[aria-current="true"]')?.getAttribute('data-key')).toBe('c');
  });

  it('d opens the detail instead of deleting', () => {
    const rows = writable<Row[]>([{ name: 'a', cpu: 1 }]);
    const onOpen = vi.fn();
    app = mount(TableHarness, { target: document.body, props: { rows, onOpen } });
    flushSync();
    selectedRowIndex.set(0);
    flushSync();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'd', bubbles: true, cancelable: true }));
    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it('marks failed rows with a danger dot and sorts problems first by default', () => {
    const rows = writable<Row[]>([
      { name: 'ok', cpu: 1 },
      { name: 'broken', cpu: 1, rank: 0 },
    ]);
    app = mount(TableHarness, {
      target: document.body,
      props: { rows, onOpen: () => {}, defaultSort: { id: 'health', dir: 'asc' } },
    });
    flushSync();
    const keys = [...document.querySelectorAll('tbody tr')].map((r) => r.getAttribute('data-key'));
    expect(keys).toEqual(['broken', 'ok']);
    const first = document.querySelector<HTMLElement>('tbody tr');
    expect(first?.dataset.problem).toBe('failed');
    expect(first?.querySelector('td .sdot')?.classList.contains('bg-danger')).toBe(true);
    // No coloured bar: only the selection draws a rail.
    expect(first?.querySelector<HTMLElement>('td')?.style.boxShadow ?? '').toBe('');
  });
});
