// List views on DataTable (PR7): pure helpers, the status-word mappings for
// types without a Rust verdict, and the polled-list loader's stale guards.
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

type Deferred<T> = { promise: Promise<T>; resolve: (v: T) => void; reject: (e: unknown) => void };
function deferred<T>(): Deferred<T> {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

const pending: Deferred<unknown>[] = [];
(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
vi.mock('@tauri-apps/api/core', () => ({
  invoke: () => {
    const d = deferred<unknown>();
    pending.push(d);
    return d.promise;
  },
}));

import {
  filterRows,
  healthCounts,
  quantityValue,
  tableStateOf,
  cellText,
  sortRows,
  type Column,
} from '../src/lib/components/ui/data-table';
import { eventHealth, pvHealth, namespaceHealth, cronJobHealth, verdictHealth } from '../src/lib/utils/status-words';
import { loadList, resourceState, currentContext } from '../src/lib/stores/kubernetes';
import { writable } from 'svelte/store';

const flush = () => new Promise((r) => setTimeout(r, 0));
/** tauriInvoke imports the API lazily, so the call lands a tick later. */
async function call(i: number): Promise<Deferred<unknown>> {
  for (let n = 0; n < 20 && !pending[i]; n++) await flush();
  return pending[i];
}

describe('filterRows', () => {
  const rows = [
    { name: 'api', hosts: ['a.example.com'], ns: 'payments' },
    { name: 'web', hosts: [], ns: 'frontend' },
  ];
  it('matches any field, case-insensitive, arrays joined', () => {
    expect(filterRows(rows, 'EXAMPLE', ['name', 'hosts']).map((r) => r.name)).toEqual(['api']);
    expect(filterRows(rows, 'front', ['name', (r) => r.ns]).map((r) => r.name)).toEqual(['web']);
  });
  it('returns every row for a blank query', () => {
    expect(filterRows(rows, '  ', ['name'])).toHaveLength(2);
  });
});

describe('cellText', () => {
  const col = (id: string): Column<Record<string, unknown>> => ({ id, label: id });
  it('joins arrays and dashes empties', () => {
    expect(cellText({ ports: ['80/TCP', '443/TCP'] }, col('ports'))).toBe('80/TCP, 443/TCP');
    expect(cellText({ ports: [] }, col('ports'))).toBe('–');
    expect(cellText({ ip: null }, col('ip'))).toBe('–');
  });
});

describe('quantityValue', () => {
  it('parses binary and decimal suffixes', () => {
    expect(quantityValue('1Gi')).toBe(2 ** 30);
    expect(quantityValue('500Mi')).toBe(500 * 2 ** 20);
    expect(quantityValue('2G')).toBe(2e9);
    expect(quantityValue('10')).toBe(10);
  });
  it('returns null for missing or unknown values', () => {
    expect(quantityValue(null)).toBeNull();
    expect(quantityValue('lots')).toBeNull();
    expect(quantityValue('5Xi')).toBeNull();
  });
  it('sorts capacities by size, not by text', () => {
    const rows = [{ name: 'a', cap: '10Gi' }, { name: 'b', cap: '500Mi' }, { name: 'c', cap: '2Ti' }];
    const cols: Column<(typeof rows)[number]>[] = [{ id: 'cap', label: 'Cap', sortKey: (r) => quantityValue(r.cap) }];
    expect(sortRows(rows, cols, { id: 'cap', dir: 'asc' }, (r) => r.name).map((r) => r.name)).toEqual(['b', 'a', 'c']);
  });
});

describe('tableStateOf', () => {
  it('maps idle and loading to loading', () => {
    expect(tableStateOf('idle')).toBe('loading');
    expect(tableStateOf('loading')).toBe('loading');
    expect(tableStateOf(undefined)).toBe('loading');
    expect(tableStateOf('forbidden')).toBe('forbidden');
    expect(tableStateOf('error')).toBe('error');
    expect(tableStateOf('ready')).toBe('ready');
  });
});

describe('status words (types without a verdict)', () => {
  it('events: Warning is warning, Normal is neutral (not healthy)', () => {
    expect(eventHealth('Warning').health).toBe('warning');
    expect(eventHealth('Normal').health).toBe('neutral');
  });
  it('PVs follow spec 11.5', () => {
    expect(pvHealth('Failed').health).toBe('failed');
    expect(pvHealth('Released').health).toBe('warning');
    expect(pvHealth('Pending').health).toBe('progressing');
    expect(pvHealth('Bound').health).toBe('healthy');
    expect(pvHealth('Available').health).toBe('neutral');
    expect(pvHealth('Weird').health).toBe('unknown');
  });
  it('namespaces: Terminating is progressing', () => {
    expect(namespaceHealth('Active').health).toBe('healthy');
    expect(namespaceHealth('Terminating').health).toBe('progressing');
  });
  it('cronjobs: suspended beats active', () => {
    expect(cronJobHealth({ suspend: true, active: 2 })).toMatchObject({ health: 'neutral', label: 'Suspended' });
    expect(cronJobHealth({ suspend: false, active: 2 })).toMatchObject({ health: 'progressing', label: '2 active' });
    expect(cronJobHealth({ suspend: false, active: 0 }).health).toBe('healthy');
  });
  it('healthCounts buckets rows for the segments', () => {
    const counts = healthCounts(['Warning', 'Normal', 'Warning'], eventHealth);
    expect(counts.warning).toBe(2);
    expect(counts.neutral).toBe(1);
    expect(counts.failed).toBe(0);
  });
  it('verdictHealth caches per row object', () => {
    const h = verdictHealth<{ verdict: { status: 'fail'; reason: string; ruleId: string } }>();
    const row = { verdict: { status: 'fail' as const, reason: 'NotReady', ruleId: 'TRI-NODE-001' } };
    expect(h(row)).toBe(h(row));
    expect(h(row)).toMatchObject({ health: 'failed', label: 'NotReady', rank: 0 });
  });
});

describe('loadList (polled lists)', () => {
  beforeEach(() => {
    pending.length = 0;
    currentContext.set('ctx-a');
  });

  it('tracks loading -> ready and keeps rows on a plain re-poll', async () => {
    const store = writable<string[]>([]);
    const state = resourceState('t-basic');
    const p = loadList('t-basic', 'get_x', store, { namespace: 'a' });
    expect(get(state).status).toBe('loading');
    (await call(0)).resolve(['one']);
    await p;
    expect(get(store)).toEqual(['one']);
    expect(get(state).status).toBe('ready');

    // Same scope: a poll doesn't flip back to loading.
    const p2 = loadList('t-basic', 'get_x', store, { namespace: 'a' });
    expect(get(state).status).toBe('ready');
    (await call(1)).resolve(['one', 'two']);
    await p2;
    expect(get(store)).toEqual(['one', 'two']);
  });

  it('drops an answer for a namespace the user already left', async () => {
    const store = writable<string[]>([]);
    const pA = loadList('t-scope', 'get_x', store, { namespace: 'a' });
    const reqA = await call(0);
    const pB = loadList('t-scope', 'get_x', store, { namespace: 'b' });
    (await call(1)).resolve(['from-b']);
    await pB;
    reqA.resolve(['from-a']); // late
    await pA;
    expect(get(store)).toEqual(['from-b']);
  });

  it('drops an answer from the previous context', async () => {
    const store = writable<string[]>(['kept']);
    const p = loadList('t-ctx', 'get_x', store);
    currentContext.set('ctx-b');
    (await call(0)).resolve(['old-cluster']);
    await p;
    expect(get(store)).toEqual(['kept']);
  });

  it('never lets an older poll overwrite a newer one', async () => {
    const store = writable<string[]>([]);
    const p1 = loadList('t-order', 'get_x', store);
    const req1 = await call(0);
    const p2 = loadList('t-order', 'get_x', store);
    (await call(1)).resolve(['new']);
    await p2;
    req1.resolve(['old']);
    await p1;
    expect(get(store)).toEqual(['new']);
  });

  it('marks 403s as forbidden, other errors as error', async () => {
    const store = writable<string[]>([]);
    const p = loadList('t-err', 'get_x', store);
    (await call(0)).reject('Kubernetes API error: 403 Forbidden');
    await p;
    expect(get(resourceState('t-err')).status).toBe('forbidden');

    const p2 = loadList('t-err2', 'get_x', store);
    (await call(1)).reject('connection refused');
    await p2;
    await flush();
    expect(get(resourceState('t-err2'))).toMatchObject({ status: 'error', error: 'connection refused' });
  });
});
