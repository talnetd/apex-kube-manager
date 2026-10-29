// Events run on listWatch (review 5): a double start (Retry, `r`, App's
// effect) can't leak a listener or duplicate rows, and only new Warning
// events from the stream notify.
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';

const listeners = new Map<string, (e: { payload: unknown }) => void>();
let watchN = 0;
const stopped: string[] = [];

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    if (cmd === 'get_events') return Promise.resolve([ev('old', 'Normal')]);
    if (cmd === 'watch_events') return Promise.resolve(`w${++watchN}`);
    if (cmd === 'stop_watch') stopped.push(String(args?.watchId));
    return Promise.resolve();
  },
}));
vi.mock('@tauri-apps/api/event', () => ({
  listen: (name: string, cb: (e: { payload: unknown }) => void) => {
    listeners.set(name, cb);
    return Promise.resolve(() => listeners.delete(name));
  },
}));

import { clusterEvents, currentContext, startEventWatch, stopEventWatch, eventList, type ClusterEventInfo } from '../src/lib/stores/kubernetes';

function ev(name: string, type: string): ClusterEventInfo {
  return {
    name,
    namespace: 'payments',
    type,
    reason: 'R',
    message: 'm',
    involved_object: 'pod-x',
    involved_kind: 'Pod',
    source: 'kubelet',
    count: 1,
    first_seen: null,
    last_seen: null,
    age: '1m',
  };
}

const flush = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  listeners.clear();
  stopped.length = 0;
  currentContext.set('ctx-1');
});

afterEach(async () => {
  await stopEventWatch();
});

describe('event watch', () => {
  it('a double start leaves exactly one listener and no duplicate rows', async () => {
    const a = startEventWatch('payments');
    const b = startEventWatch('payments');
    await Promise.all([a, b]);
    for (let i = 0; i < 5; i++) await flush();
    expect(listeners.size).toBe(1);
    expect(get(eventList.context)).toBe('ctx-1');

    const [, emit] = [...listeners.entries()][0];
    emit({ payload: { event_type: 'applied', event: ev('new', 'Warning'), timestamp: '' } });
    emit({ payload: { event_type: 'applied', event: ev('new', 'Warning'), timestamp: '' } });
    expect(get(clusterEvents).map((e) => e.name)).toEqual(['new', 'old']);
  });

  it('ignores stream events once the context has moved on', async () => {
    await startEventWatch(null);
    for (let i = 0; i < 5; i++) await flush();
    const [, emit] = [...listeners.entries()][0];
    currentContext.set('ctx-2');
    expect(get(clusterEvents)).toEqual([]);
    emit({ payload: { event_type: 'applied', event: ev('late', 'Warning'), timestamp: '' } });
    expect(get(clusterEvents)).toEqual([]);
  });
});
