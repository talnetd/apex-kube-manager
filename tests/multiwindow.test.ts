import { describe, it, expect } from 'vitest';
import { detailAccess, canRun, blockedMessage, DETAIL_ACTIONS } from '../src/lib/utils/detail-access';
import { traySummary, clusterNote, sinceLabel, type ClusterStatus } from '../src/lib/utils/tray';
import { acceptContextEvent, nextContext, effectiveContext } from '../src/lib/stores/kubernetes';

describe('detail window read-only gating', () => {
  it('is writable on its own context and before the shared one is known', () => {
    expect(detailAccess('prod', 'prod').readOnly).toBe(false);
    expect(detailAccess('prod', null).readOnly).toBe(false);
    expect(detailAccess('prod', '').readOnly).toBe(false);
  });
  it('turns read-only when the app moves to another context, and back', () => {
    const away = detailAccess('prod', 'staging');
    expect(away).toEqual({ readOnly: true, locked: 'prod', shared: 'staging' });
    expect(detailAccess('prod', 'prod').readOnly).toBe(false);
  });
  it('blocks every mutating action while read-only', () => {
    const away = detailAccess('prod', 'staging');
    const home = detailAccess('prod', 'prod');
    for (const a of DETAIL_ACTIONS) {
      expect(canRun(away, a)).toBe(false);
      expect(canRun(home, a)).toBe(true);
    }
    expect(DETAIL_ACTIONS).toEqual(['restart', 'delete', 'scale', 'exec', 'cordon', 'taint', 'edit', 'apply']);
    expect(blockedMessage(away, 'exec')).toBe('Switch back to prod to exec into this resource');
  });
  it('the main window (no locked context) is never read-only', () => {
    expect(detailAccess('', 'staging').readOnly).toBe(false);
  });
});

describe('shared context reconcile (seq-ordered)', () => {
  it('applies a newer report for another context', () => {
    expect(nextContext({ lastSeq: 1, current: 'a' }, { context: 'b', seq: 2 })).toEqual({ apply: true, lastSeq: 2 });
  });
  it('a slow local switch result loses to a newer remote switch', () => {
    // Remote switch to C (seq 5) already applied; our own switch to B returns seq 4.
    const afterRemote = nextContext({ lastSeq: 3, current: 'a' }, { context: 'c', seq: 5 });
    expect(afterRemote.apply).toBe(true);
    expect(nextContext({ lastSeq: afterRemote.lastSeq, current: 'c' }, { context: 'b', seq: 4 })).toEqual({ apply: false, lastSeq: 5 });
  });
  it('reconcile after listening catches a missed switch, and is a no-op when in step', () => {
    expect(nextContext({ lastSeq: 0, current: 'a' }, { context: 'b', seq: 7 })).toEqual({ apply: true, lastSeq: 7 });
    expect(nextContext({ lastSeq: 7, current: 'b' }, { context: 'b', seq: 7 })).toEqual({ apply: false, lastSeq: 7 });
    expect(nextContext({ lastSeq: 0, current: 'b' }, { context: 'b', seq: 3 })).toEqual({ apply: false, lastSeq: 3 });
    expect(nextContext({ lastSeq: 2, current: 'b' }, null)).toEqual({ apply: false, lastSeq: 2 });
  });
});

describe('follow-ups: A -> B -> A and the first read', () => {
  it('judges "already there" against the in-flight target, not the stale store', () => {
    // On A; a switch to B is mid-apply (currentContext still A). Then back to A.
    const current = effectiveContext('a', 'b');
    expect(current).toBe('b');
    expect(nextContext({ lastSeq: 1, current }, { context: 'a', seq: 2 })).toEqual({ apply: true, lastSeq: 2 });
    // Without an apply in flight, the store is the truth.
    expect(effectiveContext('a', null)).toBe('a');
  });
  it('accepts seq 0 on the initial read (no switch yet), but not as a later event', () => {
    expect(acceptContextEvent(0, { context: 'a', seq: 0 }, true)).toBe(true);
    expect(acceptContextEvent(0, { context: 'a', seq: 0 })).toBe(false);
    expect(nextContext({ lastSeq: 0, current: '' }, { context: 'a', seq: 0 }, true)).toEqual({ apply: true, lastSeq: 0 });
    expect(acceptContextEvent(4, { context: 'a', seq: 3 }, true)).toBe(false);
  });
});

describe('context-changed ordering', () => {
  it('accepts only newer events with a context', () => {
    expect(acceptContextEvent(0, { context: 'a', seq: 1 })).toBe(true);
    expect(acceptContextEvent(3, { context: 'a', seq: 2 })).toBe(false);
    expect(acceptContextEvent(3, { context: 'a', seq: 3 })).toBe(false);
    expect(acceptContextEvent(3, { context: '', seq: 4 })).toBe(false);
  });
});

const c = (over: Partial<ClusterStatus>): ClusterStatus => ({
  context: 'x', cluster: 'x', current: false, dot: 'green', reachable: true, critical: 0, warning: 0, detail: false, error: null, ...over,
});

describe('menu bar popover text', () => {
  it('summarises the worst state', () => {
    expect(traySummary([])).toBe('No clusters found');
    expect(traySummary([c({ dot: 'grey', reachable: null })])).toBe('Checking clusters…');
    expect(traySummary([c({}), c({ context: 'y' })])).toBe('All 2 clusters healthy');
    expect(traySummary([c({ dot: 'red' }), c({ dot: 'amber' }), c({})])).toBe('2 clusters need attention');
  });
  it('notes counts only for the current context', () => {
    expect(clusterNote(c({ reachable: null }))).toBe('checking…');
    expect(clusterNote(c({ reachable: false }))).toBe('unreachable');
    expect(clusterNote(c({}))).toBe('reachable');
    expect(clusterNote(c({ detail: true, critical: 2, warning: 1 }))).toBe('2 critical · 1 issue');
    expect(clusterNote(c({ detail: true }))).toBe('no issues');
    expect(clusterNote(c({ skipped: true, reachable: null, dot: 'grey' }))).toBe('sign-in plugin, checked when current');
  });
  it('formats the sync age', () => {
    expect(sinceLabel(null, 0)).toBeNull();
    expect(sinceLabel('2026-10-29T10:00:00Z', Date.parse('2026-10-29T10:00:04Z'))).toBe('4s');
    expect(sinceLabel('2026-10-29T10:00:00Z', Date.parse('2026-10-29T10:03:00Z'))).toBe('3m');
  });
});
