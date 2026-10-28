import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest';
import { mount, unmount, flushSync } from 'svelte';

const invoke = vi.fn((..._args: unknown[]) => Promise.resolve());
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }));

import PodList from '../src/lib/components/views/PodList.svelte';
import { pods, markSynced, type PodInfo } from '../src/lib/stores/kubernetes';
import { selectedRowIndex } from '../src/lib/stores/keyboard';

function pod(name: string, verdict: PodInfo['verdict'], status = 'Running'): PodInfo {
  return {
    name,
    namespace: 'payments',
    status,
    ready: '0/1',
    restarts: 14,
    age: '3m',
    node: 'ip-10-1',
    ip: null,
    containers: [],
    verdict,
  };
}

describe('PodList (reference view)', () => {
  let cmp: ReturnType<typeof mount> | null = null;
  const target = () => document.body;

  beforeEach(() => {
    invoke.mockClear();
    pods.set([
      pod('api-7f9c', { status: 'fail', reason: 'CrashLoopBackOff', ruleId: 'TRI-POD-003', severity: 'error' }),
      pod('web-5d8', { status: 'ok', reason: 'Running', ruleId: 'TRI-POD-000' }),
    ]);
    markSynced('pods');
    cmp = mount(PodList, { target: target() });
    flushSync();
  });

  afterEach(() => {
    if (cmp) unmount(cmp);
    cmp = null;
    document.body.innerHTML = '';
  });

  it('shows a CrashLoopBackOff pod (phase Running) as failed with its reason, first', () => {
    const rows = [...document.querySelectorAll('tbody tr')];
    expect(rows).toHaveLength(2);
    expect(rows[0].getAttribute('data-key')).toBe('payments/api-7f9c');
    const pill = rows[0].querySelector('[aria-label="CrashLoopBackOff, failed"]');
    expect(pill).not.toBeNull();
    expect(rows[1].querySelector('[aria-label="Running, healthy"]')).not.toBeNull();
  });

  it('counts it under Failing', () => {
    const failing = [...document.querySelectorAll('[role="radio"]')].find((b) => b.textContent?.includes('Failing'));
    expect(failing?.textContent).toContain('1');
    expect(failing?.hasAttribute('disabled')).toBe(false);
  });

  it('has no delete in the list; `d` opens the detail window', () => {
    expect(document.body.textContent).not.toMatch(/delete pod/i);
    expect(document.querySelector('[aria-label="Delete Pod"]')).toBeNull();
    selectedRowIndex.set(0);
    flushSync();
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'd' }));
    expect(invoke).toHaveBeenCalledWith(
      'open_resource_detail',
      expect.objectContaining({ resourceType: 'pod', name: 'api-7f9c', namespace: 'payments' })
    );
  });
});
