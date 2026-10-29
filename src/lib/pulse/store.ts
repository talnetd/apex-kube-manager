// src/lib/pulse/store.ts
// Pulse state built from stores the app already fills (watches and the
// existing metrics poll). Everything is in memory and resets on context
// switch. `initPulse()` wires the subscriptions once; they're cheap, so they
// stay for the app's lifetime and keep the history going between visits.
import { derived, get, writable, type Readable } from 'svelte/store';
import {
  currentContext,
  deployments,
  statefulsets,
  daemonsets,
  jobs,
  nodes,
  pods,
  replicasets,
  clusterEvents,
  pulseMetrics,
  nodeMetrics,
} from '../stores/kubernetes';
import { MetricHistory, type Sample } from './history';
import { computeKpis, type ActivityItem, type PulseObject } from './model';
import {
  diffNodes,
  diffRestarts,
  diffRollouts,
  diffScale,
  diffStatefulSetScale,
  mergeActivity,
  warningEvents,
} from './activity';

// ---------- Workloads and nodes as one shape ----------

export const workloads: Readable<PulseObject[]> = derived(
  [deployments, statefulsets, daemonsets, jobs],
  ([$d, $s, $ds, $j]) => [
    ...$d.map((x) => ({ kind: 'deployment' as const, name: x.name, namespace: x.namespace, verdict: x.verdict, ready: x.ready })),
    ...$s.map((x) => ({ kind: 'statefulset' as const, name: x.name, namespace: x.namespace, verdict: x.verdict, ready: x.ready })),
    ...$ds.map((x) => ({ kind: 'daemonset' as const, name: x.name, namespace: x.namespace, verdict: x.verdict, ready: `${x.ready}/${x.desired}` })),
    ...$j.map((x) => ({ kind: 'job' as const, name: x.name, namespace: x.namespace, verdict: x.verdict, ready: x.completions })),
  ]
);

export const nodeObjects: Readable<PulseObject[]> = derived(nodes, ($n) =>
  $n.map((x) => ({ kind: 'node' as const, name: x.name, namespace: null, verdict: x.verdict }))
);

export const kpis = derived(workloads, ($w) => computeKpis($w));

// ---------- History (ring buffers, sampled on the metrics poll) ----------

export const history = {
  cpu: new MetricHistory(),
  mem: new MetricHistory(),
  disk: new MetricHistory(),
  health: new MetricHistory(),
  workloads: new MetricHistory(),
};

/** Bumps on every sample so views re-read the buffers. */
export const historyTick = writable(0);

export function lastHour(name: keyof typeof history, now = Date.now()): Sample[] {
  return history[name].lastHour(now);
}

/** Worst node disk usage, percent, or null without kubelet stats. */
export function worstDisk(metrics: Record<string, { disk_usage: number | null; disk_capacity: number | null }>): number | null {
  let worst: number | null = null;
  for (const m of Object.values(metrics)) {
    if (m.disk_usage == null || !m.disk_capacity) continue;
    const pct = (m.disk_usage / m.disk_capacity) * 100;
    if (worst === null || pct > worst) worst = pct;
  }
  return worst;
}

function sample(now = Date.now()) {
  const m = get(pulseMetrics);
  if (!m) return;
  const pct = (used: number | null, of: number) => (m.metrics_available && used != null && of > 0 ? Math.min(100, (used / of) * 100) : null);
  history.cpu.add(pct(m.cpu_usage, m.cpu_allocatable), now);
  history.mem.add(pct(m.memory_usage, m.memory_allocatable), now);
  history.disk.add(worstDisk(get(nodeMetrics)), now);
  const k = get(kpis);
  // Workload lists load one by one; sampling mid-load would read as a huge
  // swing in the 1h deltas. Start once every list has answered.
  if (k.total && loaded.size === WORKLOAD_LISTS) {
    history.health.add(k.healthPct, now);
    history.workloads.add(k.total, now);
  }
  historyTick.update((n) => n + 1);
}

// ---------- Activity feed ----------

export const activity = writable<ActivityItem[]>([]);

/** Issue start fallback when a verdict has no `since`: first time Pulse saw it failing. */
const firstSeenAt = new Map<string, number>();
export function firstSeen(key: string): number {
  let t = firstSeenAt.get(key);
  if (t === undefined) {
    t = Date.now();
    firstSeenAt.set(key, t);
  }
  return t;
}

/** Changes Apex made itself (restart, scale from Pulse) show up "by Apex". */
export function recordAction(item: Omit<ActivityItem, 'id' | 't' | 'tone'> & { t?: number }) {
  const t = item.t ?? Date.now();
  activity.update((f) => mergeActivity(f, [{ ...item, t, id: `action/${t}/${item.text}`, tone: 'apex', by: 'Apex' }], t));
}

function push(items: ActivityItem[]) {
  if (items.length) activity.update((f) => mergeActivity(f, items, Date.now()));
}

let started = false;
let snap: {
  deployments: Parameters<typeof diffScale>[1] | null;
  statefulsets: Parameters<typeof diffStatefulSetScale>[1] | null;
  replicasets: Parameters<typeof diffRollouts>[1] | null;
  pods: Parameters<typeof diffRestarts>[1] | null;
  nodes: Parameters<typeof diffNodes>[1] | null;
} = { deployments: null, statefulsets: null, replicasets: null, pods: null, nodes: null };

/** Workload lists that have emitted since the last reset (not counting the initial subscribe call). */
const WORKLOAD_LISTS = 4;
const loaded = new Set<string>();
function trackLoaded(name: string, store: Readable<unknown>) {
  let first = true;
  store.subscribe(() => {
    if (first) {
      first = false;
      return;
    }
    loaded.add(name);
  });
}

function reset() {
  loaded.clear();
  for (const h of Object.values(history)) h.clear();
  historyTick.update((n) => n + 1);
  activity.set([]);
  firstSeenAt.clear();
  snap = { deployments: null, statefulsets: null, replicasets: null, pods: null, nodes: null };
}

/** An empty list right after a switch or a watch restart is "not loaded", not "everything deleted". */
const baseline = <T>(prev: readonly T[] | null, next: readonly T[]): readonly T[] | null =>
  prev && prev.length ? prev : next.length ? null : prev;

export function initPulse() {
  if (started) return;
  started = true;

  let ctx = get(currentContext);
  currentContext.subscribe((c) => {
    if (c !== ctx) {
      ctx = c;
      reset();
    }
  });

  deployments.subscribe((next) => {
    const prev = baseline(snap.deployments, next);
    push(diffScale(prev, next, Date.now()));
    snap.deployments = next;
  });
  statefulsets.subscribe((next) => {
    const prev = baseline(snap.statefulsets, next);
    push(diffStatefulSetScale(prev, next, Date.now()));
    snap.statefulsets = next;
  });
  replicasets.subscribe((next) => {
    const prev = baseline(snap.replicasets, next);
    push(diffRollouts(prev, next, Date.now()));
    snap.replicasets = next;
  });
  pods.subscribe((next) => {
    const prev = baseline(snap.pods, next);
    push(diffRestarts(prev, next, Date.now()));
    snap.pods = next;
  });
  nodes.subscribe((next) => {
    const prev = baseline(snap.nodes, next);
    push(diffNodes(prev, next, Date.now()));
    snap.nodes = next;
  });
  clusterEvents.subscribe((evs) => push(warningEvents(evs, Date.now())));

  trackLoaded('deployments', deployments);
  trackLoaded('statefulsets', statefulsets);
  trackLoaded('daemonsets', daemonsets);
  trackLoaded('jobs', jobs);

  // One history sample per metrics poll result.
  pulseMetrics.subscribe(() => sample());
}
