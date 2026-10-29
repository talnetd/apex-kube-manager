import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  parseQuantity,
  quotaUsage,
  quotaHealth,
  formatQuotaPct,
  pdbHealth,
  encodeCrType,
  parseCrType,
  crdTarget,
  type CustomTarget,
} from '../src/lib/utils/resources';

const invoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

describe('parseQuantity', () => {
  it('handles plain, milli, binary and decimal suffixes', () => {
    expect(parseQuantity('4')).toBe(4);
    expect(parseQuantity('500m')).toBe(0.5);
    expect(parseQuantity('2Gi')).toBe(2 * 2 ** 30);
    expect(parseQuantity('512Mi')).toBe(512 * 2 ** 20);
    expect(parseQuantity('1.5k')).toBe(1500);
    expect(parseQuantity('1e3')).toBe(1000);
    expect(parseQuantity(' 10 ')).toBe(10);
  });
  it('returns null instead of a confident zero for junk', () => {
    expect(parseQuantity('abc')).toBeNull();
    expect(parseQuantity('5Xi')).toBeNull();
    expect(parseQuantity('')).toBeNull();
    expect(parseQuantity(undefined)).toBeNull();
  });
});

describe('quotaUsage / quotaHealth', () => {
  it('computes percent per hard limit, fullest first', () => {
    const rows = quotaUsage({ 'requests.cpu': '4', pods: '20', 'limits.memory': '8Gi' }, { 'requests.cpu': '1', pods: '19', 'limits.memory': '4Gi' });
    expect(rows.map((r) => [r.resource, Math.round(r.pct ?? -1)])).toEqual([
      ['pods', 95],
      ['limits.memory', 50],
      ['requests.cpu', 25],
    ]);
    expect(quotaHealth(rows)).toMatchObject({ health: 'warning', label: 'pods 95%' });
  });
  it('never shows a confident 0% for unreported or unparseable usage', () => {
    const rows = quotaUsage({ secrets: '10', weird: 'lots' }, { weird: '3' });
    expect(rows.every((r) => r.pct === null)).toBe(true);
    expect(rows.find((r) => r.resource === 'secrets')?.used).toBe('');
    expect(quotaHealth(rows)).toMatchObject({ health: 'neutral', label: 'Usage not reported' });
    expect(formatQuotaPct(null)).toBe('?');
  });
  it('counts any use against a 0 limit as over (failed)', () => {
    const rows = quotaUsage({ 'services.loadbalancers': '0', pods: '10' }, { 'services.loadbalancers': '1', pods: '1' });
    expect(rows[0]).toMatchObject({ resource: 'services.loadbalancers', pct: Infinity });
    expect(quotaHealth(rows)).toMatchObject({ health: 'failed', label: 'services.loadbalancers over' });
    expect(quotaUsage({ pods: '0' }, { pods: '0' })[0].pct).toBe(0);
  });
  it('is failed at 100% and neutral without limits', () => {
    expect(quotaHealth(quotaUsage({ pods: '2' }, { pods: '2' })).health).toBe('failed');
    expect(quotaHealth([]).health).toBe('neutral');
    expect(quotaHealth(quotaUsage({ pods: '10' }, { pods: '1' })).health).toBe('healthy');
  });
});

describe('pdbHealth', () => {
  const base = { allowed_disruptions: 1, current_healthy: 3, desired_healthy: 2, expected_pods: 3 };
  it('is healthy when a disruption is allowed', () => {
    expect(pdbHealth(base)).toMatchObject({ health: 'healthy', label: '1 allowed' });
  });
  it('warns when it blocks disruptions (drains would hang)', () => {
    expect(pdbHealth({ ...base, allowed_disruptions: 0, current_healthy: 2 }).health).toBe('warning');
  });
  it('fails below the budget and is neutral with no pods', () => {
    expect(pdbHealth({ ...base, allowed_disruptions: 0, current_healthy: 1 })).toMatchObject({ health: 'failed', label: '1/2 healthy' });
    expect(pdbHealth({ ...base, expected_pods: 0 }).health).toBe('neutral');
  });
});

describe('custom resource type ids', () => {
  const t: CustomTarget = { group: 'cert-manager.io', version: 'v1', kind: 'Certificate', plural: 'certificates', namespaced: true };
  it('round-trips', () => {
    const id = encodeCrType(t);
    expect(id).toBe('cr|cert-manager.io|v1|certificates|Certificate|n');
    expect(parseCrType(id)).toEqual(t);
    expect(parseCrType(encodeCrType({ ...t, group: '', namespaced: false }))).toEqual({ ...t, group: '', namespaced: false });
  });
  it('rejects built-in kinds and malformed ids', () => {
    expect(parseCrType('pod')).toBeNull();
    expect(parseCrType('cr|a|v1|x')).toBeNull();
    expect(parseCrType('cr|a|v1|xs|X|q')).toBeNull();
  });
  it('maps a CRD row to a target', () => {
    expect(crdTarget({ group: 'g', kind: 'K', plural: 'ks', scope: 'Cluster', version: 'v2' })).toEqual({ group: 'g', version: 'v2', kind: 'K', plural: 'ks', namespaced: false });
  });
});

describe('listStore', () => {
  beforeEach(() => {
    invoke.mockReset();
    // listStore goes through kubernetes.ts's Tauri guard.
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  });

  it('passes the namespace only for namespaced lists and reports forbidden', async () => {
    const { currentContext } = await import('../src/lib/stores/kubernetes');
    const { listStore } = await import('../src/lib/stores/resources');
    currentContext.set('ctx-a');
    const ns = listStore<{ name: string }>('get_roles', true);
    const cl = listStore<{ name: string }>('get_cluster_roles', false);
    invoke.mockResolvedValueOnce([{ name: 'r' }]).mockRejectedValueOnce(new Error('403 Forbidden'));
    await ns.load('team-a');
    await cl.load('team-a');
    expect(invoke).toHaveBeenNthCalledWith(1, 'get_roles', { namespace: 'team-a' });
    expect(invoke).toHaveBeenNthCalledWith(2, 'get_cluster_roles', {});
    expect(get(ns.items)).toEqual([{ name: 'r' }]);
    expect(get(cl.state).status).toBe('forbidden');
  });

  it('skips poll ticks while a load is in flight, so a slow answer still lands', async () => {
    const { currentContext } = await import('../src/lib/stores/kubernetes');
    const { listStore } = await import('../src/lib/stores/resources');
    currentContext.set('ctx-a');
    const s = listStore<{ name: string }>('get_roles', true);
    let resolveSlow!: (v: unknown) => void;
    invoke.mockReturnValueOnce(new Promise((r) => (resolveSlow = r)));
    const slow = s.load(null);
    await s.load(null, {}, { poll: true }); // tick during the slow request: skipped
    expect(invoke).toHaveBeenCalledTimes(1);
    resolveSlow([{ name: 'slow' }]);
    await slow;
    expect(get(s.items)).toEqual([{ name: 'slow' }]);
    invoke.mockResolvedValueOnce([{ name: 'next' }]);
    await s.load(null, {}, { poll: true }); // idle again: the tick runs
    expect(get(s.items)).toEqual([{ name: 'next' }]);
  });

  it('honours a scope override (cluster-scoped custom resources)', async () => {
    const { currentContext } = await import('../src/lib/stores/kubernetes');
    const { listStore } = await import('../src/lib/stores/resources');
    currentContext.set('ctx-a');
    const s = listStore<{ name: string }>('get_custom_resources', true);
    invoke.mockResolvedValueOnce([]);
    await s.load('team-a', { target: { kind: 'K' } }, { namespaced: false });
    expect(invoke).toHaveBeenCalledWith('get_custom_resources', { target: { kind: 'K' } });
  });

  it('drops a stale answer after a newer load or a context switch, and clears on switch', async () => {
    const { currentContext } = await import('../src/lib/stores/kubernetes');
    const { listStore } = await import('../src/lib/stores/resources');
    currentContext.set('ctx-a');
    const s = listStore<{ name: string }>('get_roles', true);
    let resolveOld!: (v: unknown) => void;
    invoke.mockReturnValueOnce(new Promise((r) => (resolveOld = r))).mockResolvedValueOnce([{ name: 'new' }]);
    const old = s.load(null);
    await s.load(null);
    resolveOld([{ name: 'old' }]);
    await old;
    expect(get(s.items)).toEqual([{ name: 'new' }]);

    let resolveLate!: (v: unknown) => void;
    invoke.mockReturnValueOnce(new Promise((r) => (resolveLate = r)));
    const late = s.load(null);
    currentContext.set('ctx-b');
    expect(get(s.items)).toEqual([]);
    resolveLate([{ name: 'from-a' }]);
    await late;
    expect(get(s.items)).toEqual([]);
  });
});
