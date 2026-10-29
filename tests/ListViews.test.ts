// Migrated list views (PR7): same DataTable behaviour as Pods, no
// destructive actions on rows, narrow-window column hiding, and the
// NodeList selection invariant (anchored by name across usage re-sorts).
import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { mount, unmount, flushSync } from 'svelte';
import { get, writable, readable } from 'svelte/store';

const invoke = vi.fn((..._args: unknown[]) => Promise.resolve());
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }));

import DeploymentList from '../src/lib/components/views/DeploymentList.svelte';
import NodeList from '../src/lib/components/views/NodeList.svelte';
import SecretList from '../src/lib/components/views/SecretList.svelte';
import EventList from '../src/lib/components/views/EventList.svelte';
import ConfigMapList from '../src/lib/components/views/ConfigMapList.svelte';
import ListView from '../src/lib/components/ui/ListView.svelte';
import SecretCreator from '../src/lib/components/ui/SecretCreator.svelte';
import {
  deployments,
  nodes,
  nodeMetrics,
  secrets,
  clusterEvents,
  configmaps,
  markSynced,
  markFailed,
  currentContext,
  type DeploymentInfo,
  type NodeInfo,
  type ClusterEventInfo,
} from '../src/lib/stores/kubernetes';
import { selectedRowIndex, keyboardNavActive, totalRows, rowNavPaused, moveDown } from '../src/lib/stores/keyboard';
import type { Verdict } from '../src/lib/utils/health';

const ok: Verdict = { status: 'ok', reason: 'Ready', ruleId: 'TRI-X-000' };

function deployment(name: string, verdict: Verdict = ok): DeploymentInfo {
  return { name, namespace: 'payments', ready: '1/1', desired: 1, up_to_date: 1, available: 1, age: '2d', verdict };
}

function node(name: string, verdict: Verdict = ok): NodeInfo {
  return {
    name,
    verdict,
    status: 'Ready',
    roles: ['worker'],
    age: '10d',
    version: 'v1.36.0',
    internal_ip: '10.0.0.1',
    os_image: 'Bottlerocket',
    kernel: '6.1',
    container_runtime: 'containerd://2.0',
    taints: [],
    cpu_allocatable: 1000,
    memory_allocatable: 1024 ** 3,
  };
}

function event(name: string, type: string, age: string): ClusterEventInfo {
  return {
    name,
    namespace: 'payments',
    type,
    reason: `R-${name}`,
    message: `message ${name}`,
    involved_object: `pod-${name}`,
    involved_kind: 'Pod',
    source: 'kubelet',
    count: 1,
    first_seen: null,
    last_seen: null,
    age,
  };
}

const key = (k: string) => window.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true }));
const rowKeys = () => [...document.querySelectorAll('tbody tr')].map((r) => r.getAttribute('data-key'));
const headers = () => [...document.querySelectorAll('thead th')].map((th) => th.textContent?.trim());

let cmp: ReturnType<typeof mount> | null = null;
function render(c: Parameters<typeof mount>[0]) {
  cmp = mount(c, { target: document.body });
  flushSync();
}

beforeEach(() => {
  invoke.mockClear();
  // No context: views don't start watches or polls in tests.
  currentContext.set('');
  Object.defineProperty(window, 'innerWidth', { configurable: true, value: 1400 });
});

afterEach(() => {
  if (cmp) unmount(cmp);
  cmp = null;
  document.body.innerHTML = '';
});

describe('DeploymentList', () => {
  beforeEach(() => {
    deployments.set([
      deployment('web'),
      deployment('api', { status: 'fail', reason: '0/3 ready', ruleId: 'TRI-DEP-001', severity: 'error' }),
    ]);
    markSynced('deployments');
  });

  it('puts the failing deployment first with its verdict reason', () => {
    render(DeploymentList);
    expect(rowKeys()).toEqual(['payments/api', 'payments/web']);
    expect(document.querySelector('tbody tr [aria-label="0/3 ready, failed"]')).not.toBeNull();
  });

  it('has no scale or restart on rows; `s` and `d` open the detail window', () => {
    render(DeploymentList);
    expect(document.querySelector('tbody button')).toBeNull();
    selectedRowIndex.set(0);
    flushSync();
    key('s');
    key('d');
    const opened = invoke.mock.calls.filter((c) => c[0] === 'open_resource_detail');
    expect(opened).toHaveLength(2);
    expect(opened[0][1]).toMatchObject({ resourceType: 'deployment', name: 'api', namespace: 'payments' });
    expect(invoke.mock.calls.some((c) => /scale|restart/.test(String(c[0])))).toBe(false);
  });

  it('filters by segment', () => {
    render(DeploymentList);
    const failing = [...document.querySelectorAll<HTMLButtonElement>('[role="radio"]')].find((b) =>
      b.textContent?.includes('Failing')
    );
    failing!.click();
    flushSync();
    expect(rowKeys()).toEqual(['payments/api']);
  });

  it('hides low-priority columns in a side-panel window', () => {
    Object.defineProperty(window, 'innerWidth', { configurable: true, value: 500 });
    render(DeploymentList);
    expect(headers()).toEqual(['Name', 'Status', 'Ready', 'Age']);
  });

  it('shows every column at full width', () => {
    render(DeploymentList);
    expect(headers()).toEqual(['Name', 'Namespace', 'Status', 'Ready', 'Up-to-date', 'Available', 'Age']);
  });
});

describe('ConfigMapList states', () => {
  it('shows the forbidden state instead of an empty table', () => {
    configmaps.set([]);
    markFailed('configmaps', 'configmaps is forbidden: User cannot list resource (403)');
    render(ConfigMapList);
    expect(document.body.textContent).toContain('No access to configmaps');
  });

  it('shows the filtered-empty state with Clear filter', () => {
    configmaps.set([{ name: 'app-config', namespace: 'payments', data_count: 2, age: '1d' }]);
    markSynced('configmaps');
    render(ConfigMapList);
    const input = document.querySelector<HTMLInputElement>('input[type="text"]')!;
    input.value = 'nope';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(document.body.textContent).toContain('No configmaps match "nope"');
    // The filter is remembered per view for the session: reset it.
    input.value = '';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
  });
});

describe('NodeList', () => {
  it('keeps the selection on the same node when usage re-sorts the rows', () => {
    nodes.set([node('a'), node('b'), node('c')]);
    nodeMetrics.set({
      a: { name: 'a', cpu_usage: 100, memory_usage: null, disk_usage: null, disk_capacity: null },
      b: { name: 'b', cpu_usage: 200, memory_usage: null, disk_usage: null, disk_capacity: null },
      c: { name: 'c', cpu_usage: 300, memory_usage: null, disk_usage: null, disk_capacity: null },
    });
    markSynced('nodes');
    render(NodeList);

    // Sort by CPU ascending: a, b, c.
    const cpuHeader = [...document.querySelectorAll<HTMLButtonElement>('thead button')].find(
      (b) => b.textContent?.trim() === 'CPU'
    )!;
    cpuHeader.click();
    flushSync();
    expect(rowKeys()).toEqual(['a', 'b', 'c']);

    keyboardNavActive.set(true);
    selectedRowIndex.set(0); // "a"
    flushSync();

    // Next poll: a is now the busiest node and sorts last.
    nodeMetrics.set({
      a: { name: 'a', cpu_usage: 900, memory_usage: null, disk_usage: null, disk_capacity: null },
      b: { name: 'b', cpu_usage: 200, memory_usage: null, disk_usage: null, disk_capacity: null },
      c: { name: 'c', cpu_usage: 300, memory_usage: null, disk_usage: null, disk_capacity: null },
    });
    flushSync();
    expect(rowKeys()).toEqual(['b', 'c', 'a']);
    expect(get(selectedRowIndex)).toBe(2);
    expect(document.querySelector('tr[aria-current="true"]')?.getAttribute('data-key')).toBe('a');

    key('Enter');
    expect(invoke).toHaveBeenCalledWith('open_resource_detail', expect.objectContaining({ resourceType: 'node', name: 'a' }));
  });
});

describe('SecretList', () => {
  it('offers Edit on rows but never Delete', () => {
    secrets.set([{ name: 'db-creds', namespace: 'payments', secret_type: 'Opaque', data_count: 2, age: '3d' }]);
    markSynced('secrets');
    render(SecretList);
    expect(document.querySelector('tbody [aria-label="Edit"]')).not.toBeNull();
    expect(document.querySelector('tbody [aria-label*="Delete"]')).toBeNull();
    selectedRowIndex.set(0);
    flushSync();
    key('d');
    expect(invoke.mock.calls.some((c) => String(c[0]).startsWith('delete'))).toBe(false);
    // Like every list: `d` opens the detail window (review 6).
    expect(invoke).toHaveBeenCalledWith('open_resource_detail', expect.objectContaining({ resourceType: 'secret', name: 'db-creds' }));
  });
});

describe('EventList', () => {
  it('sorts warnings first, then newest', () => {
    clusterEvents.set([event('n-new', 'Normal', '1m'), event('w-old', 'Warning', '2h'), event('w-new', 'Warning', '5m')]);
    markSynced('events');
    render(EventList);
    expect(rowKeys()).toEqual(['payments/w-new', 'payments/w-old', 'payments/n-new']);
    expect(document.querySelector('[aria-label="Normal, neutral"]')).not.toBeNull();
  });
});

describe('stale data (review 4)', () => {
  it('keeps rows on a failed refresh, with a banner, and they stay openable', () => {
    configmaps.set([{ name: 'app-config', namespace: 'payments', data_count: 2, age: '1d' }]);
    markSynced('configmaps');
    markFailed('configmaps', 'connection refused');
    render(ConfigMapList);
    expect(rowKeys()).toEqual(['payments/app-config']);
    expect(document.querySelector('[role="status"]')?.textContent).toContain("Couldn't refresh configmaps");
    selectedRowIndex.set(0);
    flushSync();
    key('Enter');
    expect(invoke).toHaveBeenCalledWith('open_resource_detail', expect.objectContaining({ name: 'app-config' }));
  });
});

describe('context guard (review 1)', () => {
  function fakeSource(rowsContext: string) {
    return {
      name: 'fake',
      items: writable([{ name: 'db', namespace: 'payments' }]),
      state: readable({ status: 'ready' as const }),
      context: readable(rowsContext),
      namespaced: true,
      reset() {},
    };
  }
  const props = (rowsContext: string) => ({
    title: 'Things',
    viewId: 'things',
    itemLabel: 'things',
    source: fakeSource(rowsContext),
    columns: [{ id: 'name', label: 'Name', sortKey: 'name', primary: true }],
    rowKey: (r: { name: string }) => r.name,
    resourceType: 'secret',
    shortcuts: { e: editSpy },
  });
  const editSpy = vi.fn();

  it('refuses to open or act on rows loaded from another context', () => {
    currentContext.set('ctx-b');
    cmp = mount(ListView, { target: document.body, props: props('ctx-a') });
    flushSync();
    selectedRowIndex.set(0);
    flushSync();
    key('Enter');
    key('e');
    expect(invoke).not.toHaveBeenCalled();
    expect(editSpy).not.toHaveBeenCalled();
  });

  it('opens with the context the rows came from', () => {
    currentContext.set('ctx-b');
    cmp = mount(ListView, { target: document.body, props: props('ctx-b') });
    flushSync();
    selectedRowIndex.set(0);
    flushSync();
    key('Enter');
    expect(invoke).toHaveBeenCalledWith('open_resource_detail', expect.objectContaining({ name: 'db', context: 'ctx-b' }));
  });

  it('closes an open Secret editor when the context changes (external kubectx)', () => {
    currentContext.set('ctx-a');
    const onclose = vi.fn();
    cmp = mount(SecretCreator, {
      target: document.body,
      props: { show: true, onclose, context: 'ctx-a', editData: { name: 'db', namespace: 'payments', secret_type: 'Opaque', data: {} } },
    });
    flushSync();
    expect(onclose).not.toHaveBeenCalled();
    currentContext.set('ctx-b');
    flushSync();
    expect(onclose).toHaveBeenCalled();
  });
});

describe('keyboard while an editor is open (review 7)', () => {
  it('pauses j/k movement and resumes after', () => {
    totalRows.set(3);
    selectedRowIndex.set(0);
    rowNavPaused.set(true);
    moveDown();
    expect(get(selectedRowIndex)).toBe(0);
    rowNavPaused.set(false);
    moveDown();
    expect(get(selectedRowIndex)).toBe(1);
  });
});
