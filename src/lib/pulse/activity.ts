// src/lib/pulse/activity.ts
// Activity feed (spec 6.5): changes derived by diffing data the app already
// watches or polls. Pure: each differ takes the previous and next snapshot
// and returns new feed items. The first snapshot is a baseline, never a change.
import type { ActivityItem } from './model';
import { KIND_SHORT } from './model';
import { parseAge } from '../utils/sort';
import { HOUR_MS } from './history';

interface Verdicted {
  verdict: { status: string; reason: string };
}

export interface DeploymentLike extends Verdicted {
  name: string;
  namespace: string;
  ready: string;
  /** spec.replicas. `ready` counts surge pods, so a rollout would look like scaling. */
  desired: number;
}
export interface StatefulSetLike extends Verdicted {
  name: string;
  namespace: string;
  replicas: number;
}
export interface ReplicaSetLike {
  name: string;
  namespace: string;
  owner: string | null;
  desired: number;
  age: string;
}
export interface PodLike {
  name: string;
  namespace: string;
  restarts: number;
}
export interface NodeLike extends Verdicted {
  name: string;
  taints: { key: string; effect: string }[];
}
export interface EventLike {
  name: string;
  namespace: string;
  type: string;
  reason: string;
  message: string;
  involved_object: string;
  involved_kind: string;
  count: number;
  last_seen: string | null;
  age: string;
}

export function diffScale(
  prev: readonly DeploymentLike[] | null,
  next: readonly DeploymentLike[],
  now: number
): ActivityItem[] {
  if (!prev) return [];
  const before = new Map(prev.map((d) => [`${d.namespace}/${d.name}`, d.desired]));
  const out: ActivityItem[] = [];
  for (const d of next) {
    const a = before.get(`${d.namespace}/${d.name}`);
    const b = d.desired;
    if (a == null || a === b) continue;
    out.push({
      id: `scale/deployment/${d.namespace}/${d.name}/${now}`,
      t: now,
      kind: 'scale',
      tone: 'neutral',
      objectKind: 'deployment',
      target: { kind: 'deployment', name: d.name, namespace: d.namespace },
      text: `Scaled ${KIND_SHORT.deployment}/${d.name} ${a} → ${b}`,
      detail: d.namespace,
    });
  }
  return out;
}

export function diffStatefulSetScale(
  prev: readonly StatefulSetLike[] | null,
  next: readonly StatefulSetLike[],
  now: number
): ActivityItem[] {
  if (!prev) return [];
  const before = new Map(prev.map((s) => [`${s.namespace}/${s.name}`, s.replicas]));
  const out: ActivityItem[] = [];
  for (const s of next) {
    const a = before.get(`${s.namespace}/${s.name}`);
    if (a == null || a === s.replicas) continue;
    out.push({
      id: `scale/statefulset/${s.namespace}/${s.name}/${now}`,
      t: now,
      kind: 'scale',
      tone: 'neutral',
      objectKind: 'statefulset',
      target: { kind: 'statefulset', name: s.name, namespace: s.namespace },
      text: `Scaled ${KIND_SHORT.statefulset}/${s.name} ${a} → ${s.replicas}`,
      detail: s.namespace,
    });
  }
  return out;
}

/**
 * A rollout shows up as a new ReplicaSet under a Deployment that already had
 * one (each revision gets its own ReplicaSet).
 */
export function diffRollouts(
  prev: readonly ReplicaSetLike[] | null,
  next: readonly ReplicaSetLike[],
  now: number
): ActivityItem[] {
  if (!prev) return [];
  const known = new Set(prev.map((r) => `${r.namespace}/${r.name}`));
  const owners = new Set(prev.filter((r) => r.owner).map((r) => `${r.namespace}/${r.owner}`));
  const out: ActivityItem[] = [];
  for (const r of next) {
    if (!r.owner || known.has(`${r.namespace}/${r.name}`)) continue;
    if (!owners.has(`${r.namespace}/${r.owner}`)) continue; // a brand-new Deployment, not a rollout
    const rev = r.name.startsWith(`${r.owner}-`) ? r.name.slice(r.owner.length + 1) : r.name;
    out.push({
      id: `rollout/${r.namespace}/${r.name}`,
      // When the ReplicaSet was created, not when the 30s poll noticed it,
      // so it can still precede the issue it caused.
      t: Math.min(now, now - parseAge(r.age) * 1000),
      kind: 'rollout',
      tone: 'neutral',
      objectKind: 'replicaset',
      target: { kind: 'deployment', name: r.owner, namespace: r.namespace },
      text: `Rollout ${KIND_SHORT.deployment}/${r.owner} → ${rev}`,
      detail: `${r.namespace} · new ReplicaSet ${r.name}`,
    });
  }
  return out;
}

/**
 * Container restarts, one item per pod (updated in place by the store), so a
 * crash-looping pod doesn't flood the feed.
 */
export function diffRestarts(prev: readonly PodLike[] | null, next: readonly PodLike[], now: number): ActivityItem[] {
  if (!prev) return [];
  const before = new Map(prev.map((p) => [`${p.namespace}/${p.name}`, p.restarts]));
  const out: ActivityItem[] = [];
  for (const p of next) {
    const a = before.get(`${p.namespace}/${p.name}`);
    if (a == null || p.restarts <= a) continue;
    out.push({
      id: `restart/${p.namespace}/${p.name}`,
      t: now,
      kind: 'restart',
      tone: 'warn',
      objectKind: 'pod',
      target: null,
      text: `Container restarted in pod/${p.name}`,
      detail: `${p.namespace} · ${p.restarts} restarts`,
    });
  }
  return out;
}

const CORDON_TAINT = 'node.kubernetes.io/unschedulable';

export function diffNodes(prev: readonly NodeLike[] | null, next: readonly NodeLike[], now: number): ActivityItem[] {
  if (!prev) return [];
  const before = new Map(prev.map((n) => [n.name, n]));
  const out: ActivityItem[] = [];
  for (const n of next) {
    const p = before.get(n.name);
    if (!p) continue;
    const target = { kind: 'node' as const, name: n.name, namespace: null };
    if (p.verdict.status === 'ok' && n.verdict.status === 'fail') {
      out.push({ id: `node/${n.name}/${now}/down`, t: now, kind: 'node', tone: 'bad', objectKind: 'node', target, text: `node/${n.name} ${n.verdict.reason || 'NotReady'}`, detail: 'Node condition changed' });
    } else if (p.verdict.status !== 'ok' && n.verdict.status === 'ok') {
      out.push({ id: `node/${n.name}/${now}/up`, t: now, kind: 'node', tone: 'neutral', objectKind: 'node', target, text: `node/${n.name} Ready again`, detail: 'Node condition changed' });
    }
    const wasCordoned = p.taints.some((t) => t.key === CORDON_TAINT);
    const isCordoned = n.taints.some((t) => t.key === CORDON_TAINT);
    if (!wasCordoned && isCordoned) {
      out.push({ id: `node/${n.name}/${now}/cordon`, t: now, kind: 'node', tone: 'warn', objectKind: 'node', target, text: `Cordoned node/${n.name}`, detail: 'Scheduling disabled' });
    } else if (wasCordoned && !isCordoned) {
      out.push({ id: `node/${n.name}/${now}/uncordon`, t: now, kind: 'node', tone: 'neutral', objectKind: 'node', target, text: `Uncordoned node/${n.name}`, detail: 'Scheduling enabled' });
    }
  }
  return out;
}

/** When an event last happened: `last_seen` if set, else now minus its age. */
export function eventTime(e: Pick<EventLike, 'last_seen' | 'age'>, now: number): number {
  const t = e.last_seen ? Date.parse(e.last_seen) : NaN;
  return Number.isFinite(t) ? t : now - parseAge(e.age) * 1000;
}

/**
 * Warning events from the last hour. Keyed per event, so a repeating event
 * moves to the top with its new count instead of adding rows.
 */
export function warningEvents(events: readonly EventLike[], now: number): ActivityItem[] {
  const out: ActivityItem[] = [];
  for (const e of events) {
    if (e.type !== 'Warning') continue;
    const t = eventTime(e, now);
    if (now - t > HOUR_MS) continue;
    const kind = e.involved_kind === 'Node' ? 'node' : e.involved_kind.toLowerCase();
    out.push({
      id: `event/${e.namespace}/${e.name}`,
      t,
      kind: 'warning',
      tone: 'warn',
      objectKind: 'event',
      target: null,
      text: `${e.reason} on ${kind}/${e.involved_object}`,
      detail: `${e.message}${e.count > 1 ? ` (×${e.count})` : ''}`,
    });
  }
  return out;
}

const SYMPTOMS = new Set(['restart', 'warning']);

/**
 * Merge new items into the feed: same id replaces, newest first, last hour
 * only. Causes (rollouts, scaling, node changes, actions) and symptoms
 * (restarts, Warning events) are capped separately, so a noisy cluster's
 * warnings can't push out the change that explains them.
 */
export function mergeActivity(feed: readonly ActivityItem[], items: readonly ActivityItem[], now: number, maxPerClass = 150): ActivityItem[] {
  const byId = new Map(feed.map((a) => [a.id, a]));
  for (const it of items) byId.set(it.id, it);
  const all = [...byId.values()].filter((a) => now - a.t <= HOUR_MS).sort((a, b) => b.t - a.t);
  let causes = 0;
  let symptoms = 0;
  return all.filter((a) => (SYMPTOMS.has(a.kind) ? ++symptoms : ++causes) <= maxPerClass);
}
