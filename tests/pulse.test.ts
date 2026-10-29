import { describe, it, expect } from 'vitest';
import { RingBuffer, MetricHistory, windowed, delta, HOUR_MS } from '../src/lib/pulse/history';
import {
  computeKpis,
  reachabilityPct,
  buildIssues,
  findTrigger,
  podsFor,
  ownsPodName,
  sevOf,
  shortDuration,
  TRIGGER_WINDOW_MS,
  type ActivityItem,
  type PulseObject,
} from '../src/lib/pulse/model';
import { diffRollouts, diffScale, diffRestarts, diffNodes, mergeActivity, desiredOf, warningEvents } from '../src/lib/pulse/activity';
import type { Verdict } from '../src/lib/utils/health';

const ok: Verdict = { status: 'ok', reason: 'Available', ruleId: 'ok' };
const fail = (reason: string, since?: string, severity: Verdict['severity'] = 'critical'): Verdict => ({ status: 'fail', reason, ruleId: 'r', severity, since });
const warn = (reason: string): Verdict => ({ status: 'warn', reason, ruleId: 'r', severity: 'warning' });

describe('RingBuffer', () => {
  it('keeps the newest items, oldest first', () => {
    const r = new RingBuffer<number>(3);
    [1, 2, 3, 4, 5].forEach((n) => r.push(n));
    expect(r.toArray()).toEqual([3, 4, 5]);
    expect(r.length).toBe(3);
    expect(r.last()).toBe(5);
  });
  it('clears and rejects a bad capacity', () => {
    const r = new RingBuffer<number>(2);
    r.push(1);
    r.clear();
    expect(r.toArray()).toEqual([]);
    expect(r.last()).toBeUndefined();
    expect(() => new RingBuffer(0)).toThrow();
  });
});

describe('MetricHistory', () => {
  it('returns only the last hour and dedupes samples inside a second', () => {
    const h = new MetricHistory(10);
    const t0 = 1_000_000_000;
    h.add(10, t0);
    h.add(11, t0 + 500); // same second: dropped
    h.add(20, t0 + HOUR_MS / 2);
    h.add(30, t0 + HOUR_MS + 1000);
    const s = h.lastHour(t0 + HOUR_MS + 1000);
    expect(s.map((x) => x.v)).toEqual([20, 30]);
  });
  it('delta ignores nulls and needs two points', () => {
    expect(delta([{ t: 1, v: 5 }])).toBeNull();
    expect(delta([{ t: 1, v: 90 }, { t: 2, v: null }, { t: 3, v: 95.5 }])).toBeCloseTo(5.5);
    expect(windowed([{ t: 0, v: 1 }, { t: 10, v: 2 }], 10, 5)).toEqual([{ t: 10, v: 2 }]);
  });
});

describe('computeKpis', () => {
  it('counts verdict statuses; unknown only adds to the total', () => {
    const k = computeKpis([{ verdict: ok }, { verdict: ok }, { verdict: warn('1/2 ready') }, { verdict: fail('CrashLoopBackOff') }, { verdict: { status: 'unknown', reason: '', ruleId: 'u' } }]);
    expect(k).toEqual({ total: 5, healthy: 2, warning: 1, critical: 1, healthPct: 40 });
  });
  it('has no health % without workloads', () => {
    expect(computeKpis([]).healthPct).toBeNull();
  });
  it('reachability is ok over total', () => {
    expect(reachabilityPct(0, 0)).toBeNull();
    expect(reachabilityPct(199, 200)).toBeCloseTo(99.5);
  });
});

describe('buildIssues', () => {
  const objs: PulseObject[] = [
    { kind: 'deployment', name: 'web', namespace: 'a', verdict: ok },
    { kind: 'deployment', name: 'notifier', namespace: 'a', verdict: warn('1/2 ready') },
    { kind: 'statefulset', name: 'ledger', namespace: 'a', verdict: fail('OOMKilled', '2026-10-29T07:00:00Z', 'error') },
    { kind: 'node', name: 'n1', namespace: null, verdict: fail('NotReady', '2026-10-29T07:10:00Z') },
    { kind: 'deployment', name: 'checkout', namespace: 'a', verdict: fail('CrashLoopBackOff', '2026-10-29T07:05:00Z') },
  ];
  it('lists problems only, worst severity first, newest first within a severity', () => {
    const issues = buildIssues(objs, () => 0);
    expect(issues.map((i) => i.name)).toEqual(['n1', 'checkout', 'ledger', 'notifier']);
    expect(issues.map((i) => i.sev)).toEqual(['crit', 'crit', 'err', 'warn']);
  });
  it('uses first-seen time when the verdict has no since', () => {
    const issues = buildIssues(objs, () => 42);
    expect(issues.find((i) => i.name === 'notifier')?.startedAt).toBe(42);
    expect(issues.find((i) => i.name === 'ledger')?.startedAt).toBe(Date.parse('2026-10-29T07:00:00Z'));
  });
  it('maps severity, falling back on status', () => {
    expect(sevOf(ok)).toBeNull();
    expect(sevOf({ status: 'fail', reason: '', ruleId: 'r' })).toBe('err');
    expect(sevOf({ status: 'warn', reason: '', ruleId: 'r' })).toBe('warn');
  });
});

describe('podsFor', () => {
  const pods = [
    { name: 'checkout-7d9f8c-abcde', namespace: 'a', node: 'n1', verdict: ok },
    { name: 'checkout-api-7d9f8c-abcde', namespace: 'a', node: 'n2', verdict: ok },
    { name: 'checkout-7d9f8c-fghij', namespace: 'b', node: 'n1', verdict: ok },
  ];
  it('matches workload pods by name prefix in the same namespace', () => {
    expect(podsFor({ kind: 'deployment', name: 'checkout', namespace: 'a' }, pods).map((p) => p.name)).toEqual(['checkout-7d9f8c-abcde']);
  });
  it('follows each controller naming scheme', () => {
    expect(ownsPodName('statefulset', 'db', 'db-0')).toBe(true);
    expect(ownsPodName('statefulset', 'db', 'db-proxy-0')).toBe(false);
    expect(ownsPodName('job', 'report', 'report-x7k2p')).toBe(true);
  });
  it('matches node pods by node name', () => {
    expect(podsFor({ kind: 'node', name: 'n1', namespace: null }, pods)).toHaveLength(2);
  });
});

describe('findTrigger', () => {
  const start = 10_000_000;
  const item = (id: string, t: number, kind: ActivityItem['kind'], target: ActivityItem['target']): ActivityItem => ({
    id, t, kind, tone: 'neutral', objectKind: 'deployment', target, text: id, detail: '',
  });
  const checkout = { kind: 'deployment' as const, name: 'checkout', namespace: 'a' };
  const issue = { ...checkout, startedAt: start };

  it('picks the latest change to the same object in the 15 minutes before', () => {
    const t = findTrigger(issue, [
      item('old', start - TRIGGER_WINDOW_MS - 1, 'rollout', checkout),
      item('early', start - 10 * 60_000, 'rollout', checkout),
      item('late', start - 4 * 60_000, 'scale', checkout),
      item('after', start + 1000, 'rollout', checkout),
    ]);
    expect(t?.item.id).toBe('late');
    expect(t?.afterMs).toBe(4 * 60_000);
  });
  it('ignores symptoms and other objects', () => {
    const t = findTrigger(issue, [
      item('restart', start - 1000, 'restart', checkout),
      item('event', start - 1000, 'warning', checkout),
      item('other', start - 1000, 'rollout', { kind: 'deployment', name: 'web', namespace: 'a' }),
      item('other-ns', start - 1000, 'rollout', { kind: 'deployment', name: 'checkout', namespace: 'b' }),
    ]);
    expect(t).toBeNull();
  });
  it('falls back to a change on a node running the pods, but prefers the object itself', () => {
    const node = { kind: 'node' as const, name: 'n1', namespace: null };
    expect(findTrigger(issue, [item('cordon', start - 1000, 'node', node)], ['n1'])?.item.id).toBe('cordon');
    expect(findTrigger(issue, [item('cordon', start - 1000, 'node', node)], ['n2'])).toBeNull();
    const t = findTrigger(issue, [item('cordon', start - 1000, 'node', node), item('rollout', start - 5000, 'rollout', checkout)], ['n1']);
    expect(t?.item.id).toBe('rollout');
  });
});

describe('activity differs', () => {
  const now = 5_000_000;
  it('treat the first snapshot as a baseline', () => {
    expect(diffScale(null, [{ name: 'a', namespace: 'x', ready: '1/3', verdict: ok }], now)).toEqual([]);
    expect(diffRollouts(null, [{ name: 'a-1', namespace: 'x', owner: 'a', desired: 1 }], now)).toEqual([]);
  });
  it('detect scaling from the desired count', () => {
    expect(desiredOf('2/5')).toBe(5);
    const out = diffScale([{ name: 'a', namespace: 'x', ready: '3/3', verdict: ok }], [{ name: 'a', namespace: 'x', ready: '3/5', verdict: ok }], now);
    expect(out[0].text).toBe('Scaled deploy/a 3 → 5');
    expect(out[0].target).toEqual({ kind: 'deployment', name: 'a', namespace: 'x' });
  });
  it('detect a rollout as a new ReplicaSet under a known owner, not a new Deployment', () => {
    const prev = [{ name: 'a-111', namespace: 'x', owner: 'a', desired: 3 }];
    const next = [...prev, { name: 'a-222', namespace: 'x', owner: 'a', desired: 3 }, { name: 'b-333', namespace: 'x', owner: 'b', desired: 1 }];
    const out = diffRollouts(prev, next, now);
    expect(out).toHaveLength(1);
    expect(out[0].text).toBe('Rollout deploy/a → 222');
  });
  it('detect restarts and node changes', () => {
    expect(diffRestarts([{ name: 'p', namespace: 'x', restarts: 1 }], [{ name: 'p', namespace: 'x', restarts: 3 }], now)[0].detail).toContain('3 restarts');
    const n = (status: string, taints: { key: string; effect: string }[] = []) => ({ name: 'n1', taints, verdict: { status, reason: status === 'ok' ? 'Ready' : 'NotReady' } });
    expect(diffNodes([n('ok')], [n('fail')], now)[0].tone).toBe('bad');
    expect(diffNodes([n('ok')], [n('ok', [{ key: 'node.kubernetes.io/unschedulable', effect: 'NoSchedule' }])], now)[0].text).toBe('Cordoned node/n1');
  });
  it('keep Warning events from the last hour only', () => {
    const ev = (name: string, age: string, type = 'Warning') => ({ name, namespace: 'x', type, reason: 'BackOff', message: 'm', involved_object: 'p', involved_kind: 'Pod', count: 2, last_seen: null, age });
    const out = warningEvents([ev('a', '5m'), ev('b', '2h'), ev('c', '1m', 'Normal')], now);
    expect(out.map((o) => o.id)).toEqual(['event/x/a']);
    expect(out[0].t).toBe(now - 5 * 60_000);
  });
  it('merge replaces by id, sorts newest first and drops items older than an hour', () => {
    const a = { id: 'a', t: now - 1000, kind: 'restart', tone: 'warn', objectKind: 'pod', target: null, text: 'a', detail: '' } as ActivityItem;
    const old = { ...a, id: 'old', t: now - HOUR_MS - 1 };
    const merged = mergeActivity([a, old], [{ ...a, t: now }, { ...a, id: 'b', t: now - 500 }], now);
    expect(merged.map((m) => [m.id, m.t])).toEqual([['a', now], ['b', now - 500]]);
  });
});

describe('shortDuration', () => {
  it('formats compact ages', () => {
    expect(shortDuration(1000)).toBe('now');
    expect(shortDuration(45_000)).toBe('45s');
    expect(shortDuration(14 * 60_000)).toBe('14m');
    expect(shortDuration(3 * 3600_000)).toBe('3h');
    expect(shortDuration(72 * 3600_000)).toBe('3d');
  });
});
