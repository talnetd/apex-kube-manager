// Pod watch lifecycle (review findings 3 and 8): stale setups never write,
// superseded watches are stopped, and events are applied in batches.
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

type Deferred<T> = { promise: Promise<T>; resolve: (v: T) => void };
function deferred<T>(): Deferred<T> {
  let resolve!: (v: T) => void;
  const promise = new Promise<T>((r) => (resolve = r));
  return { promise, resolve };
}

const calls: { cmd: string; args?: Record<string, unknown> }[] = [];
let getPods: (ns: unknown) => Promise<unknown>;
let watchPods: (ns: unknown) => Promise<string>;
const listeners = new Map<string, (e: { payload: unknown }) => void>();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    calls.push({ cmd, args });
    if (cmd === 'get_pods') return getPods(args?.namespace);
    if (cmd === 'watch_pods') return watchPods(args?.namespace);
    return Promise.resolve();
  },
}));
vi.mock('@tauri-apps/api/event', () => ({
  listen: (name: string, cb: (e: { payload: unknown }) => void) => {
    listeners.set(name, cb);
    return Promise.resolve(() => listeners.delete(name));
  },
}));

import {
  pods,
  currentContext,
  startPodWatch,
  stopPodWatch,
  applyPodEvents,
  type PodInfo,
  type PodWatchEvent,
} from '../src/lib/stores/kubernetes';

const ok = { status: 'ok', reason: 'Running', ruleId: 'TRI-POD-000' } as const;
const pod = (name: string, namespace = 'a', reason = 'Running'): PodInfo => ({
  name,
  namespace,
  status: 'Running',
  ready: '1/1',
  restarts: 0,
  age: '1m',
  node: null,
  ip: null,
  containers: [],
  verdict: { ...ok, reason },
});
const flush = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  calls.length = 0;
  listeners.clear();
  currentContext.set('ctx-1');
  pods.set([]);
});

afterEach(async () => {
  vi.useRealTimers();
  await stopPodWatch();
});

describe('applyPodEvents', () => {
  it('updates, adds and deletes in order, last event per pod wins', () => {
    const ev = (event_type: 'applied' | 'deleted', p: PodInfo): PodWatchEvent => ({ event_type, pod: p, timestamp: '' });
    const out = applyPodEvents(
      [pod('x'), pod('y')],
      [ev('applied', pod('x', 'a', 'CrashLoopBackOff')), ev('applied', pod('z')), ev('deleted', pod('y')), ev('applied', pod('y'))]
    );
    expect(out.map((p) => `${p.name}:${p.verdict.reason}`)).toEqual(['x:CrashLoopBackOff', 'z:Running', 'y:Running']);
  });
});

describe('startPodWatch generation guard', () => {
  it('a slow first list from a superseded start never paints, and its watch is stopped', async () => {
    const slowList = deferred<PodInfo[]>();
    const slowWatch = deferred<string>();
    getPods = (ns) => (ns === 'a' ? slowList.promise : Promise.resolve([pod('b1', 'b')]));
    watchPods = (ns) => (ns === 'a' ? slowWatch.promise : Promise.resolve('watch-b'));

    const a = startPodWatch('a');
    await flush();
    await startPodWatch('b');
    expect(get(pods).map((p) => p.name)).toEqual(['b1']);

    slowList.resolve([pod('a1')]);
    slowWatch.resolve('watch-a');
    await a;
    expect(get(pods).map((p) => p.name)).toEqual(['b1']);
    // A never got as far as watching, so there's nothing of A's to stop...
    expect(calls.filter((c) => c.cmd === 'watch_pods').map((c) => c.args?.namespace)).toEqual(['b']);
  });

  it('a watch that resolves after a newer start is stopped and never listened to', async () => {
    const slowWatch = deferred<string>();
    getPods = () => Promise.resolve([]);
    watchPods = (ns) => (ns === 'a' ? slowWatch.promise : Promise.resolve('watch-b'));

    const a = startPodWatch('a');
    await flush();
    await startPodWatch('b');
    slowWatch.resolve('watch-a');
    await a;
    await flush(); // stop_watch is fire-and-forget

    expect(calls.filter((c) => c.cmd === 'watch_pods').map((c) => c.args?.namespace)).toEqual(['a', 'b']);
    expect(calls).toContainEqual({ cmd: 'stop_watch', args: { watchId: 'watch-a' } });
    expect(listeners.has('pod-watch-watch-a')).toBe(false);
    expect(listeners.has('pod-watch-watch-b')).toBe(true);
  });

  it('a stale context switch mid-list drops the result', async () => {
    const slowList = deferred<PodInfo[]>();
    getPods = () => slowList.promise;
    watchPods = () => Promise.resolve('w');
    const a = startPodWatch(null);
    await flush();
    currentContext.set('ctx-2');
    slowList.resolve([pod('old-cluster')]);
    await a;
    expect(get(pods)).toEqual([]);
  });
});

describe('watch event batching', () => {
  it('applies a burst once after 100ms; events for a superseded watch are dropped', async () => {
    getPods = () => Promise.resolve([pod('x')]);
    watchPods = () => Promise.resolve('w1');
    await startPodWatch('a');
    const emit = listeners.get('pod-watch-w1')!;

    vi.useFakeTimers();
    const updates: number[] = [];
    const unsub = pods.subscribe((p) => updates.push(p.length));
    updates.length = 0;

    for (let i = 0; i < 50; i++) emit({ payload: { event_type: 'applied', pod: pod(`p${i}`), timestamp: '' } });
    expect(get(pods)).toHaveLength(1); // nothing applied yet
    vi.advanceTimersByTime(100);
    expect(get(pods)).toHaveLength(51);
    expect(updates).toEqual([51]); // one store update for the whole burst

    await stopPodWatch();
    emit({ payload: { event_type: 'applied', pod: pod('late'), timestamp: '' } });
    vi.advanceTimersByTime(200);
    expect(get(pods).some((p) => p.name === 'late')).toBe(false);
    unsub();
  });
});
