// Frontend side of the shared triage contract: verdicts come from Rust and
// the UI only maps them (health.ts) and buckets them (kubernetes.ts).
import { describe, it, expect, vi, afterEach } from 'vitest';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { get } from 'svelte/store';
import { toDisplay, type Verdict } from '../src/lib/utils/health';
import {
  bucketByHealth,
  resourceState,
  markLoading,
  markSynced,
  markFailed,
  isForbiddenError,
  lastSyncedAt,
} from '../src/lib/stores/kubernetes';

const FIXTURES = join(__dirname, '..', 'spec', 'triage', 'fixtures');

function expectedFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    if (statSync(p).isDirectory()) return expectedFiles(p);
    return p.endsWith('.expected.json') ? [p] : [];
  });
}

const load = (rel: string): Verdict => JSON.parse(readFileSync(join(FIXTURES, rel), 'utf8'));

describe('shared fixtures through the display mapping', () => {
  const files = expectedFiles(FIXTURES);

  it('every expected verdict maps to a display state with its reason as label', () => {
    expect(files.length).toBeGreaterThanOrEqual(25);
    for (const f of files) {
      const v = JSON.parse(readFileSync(f, 'utf8')) as Verdict;
      const d = toDisplay(v);
      expect(d.label, f).toBe(v.reason);
      if (v.status === 'fail') expect(d.health, f).toBe('failed');
      if (v.status === 'warn') expect(d.health, f).toBe('warning');
      if (v.status === 'ok' && !v.tone) expect(d.health, f).toBe('healthy');
    }
  });

  it('a CrashLoopBackOff pod (phase Running) is failed, not healthy', () => {
    const d = toDisplay(load('pod/crashloop-backoff.expected.json'));
    expect(d).toMatchObject({ health: 'failed', label: 'CrashLoopBackOff', rank: 0, severity: 'error' });
  });

  it('tone drives progressing and neutral', () => {
    expect(toDisplay(load('pod/container-creating.expected.json')).health).toBe('progressing');
    expect(toDisplay(load('pod/succeeded-completed.expected.json')).health).toBe('neutral');
  });
});

describe('bucketByHealth (podsByStatus)', () => {
  const pod = (name: string, verdict: Verdict) => ({ name, status: 'Running', verdict });
  const rows = [
    pod('crash', { status: 'fail', reason: 'CrashLoopBackOff', ruleId: 'TRI-POD-003', severity: 'error' }),
    pod('slow', { status: 'warn', reason: '1/2 ready', ruleId: 'TRI-POD-007', severity: 'warning' }),
    pod('new', { status: 'ok', reason: 'Pending', ruleId: 'TRI-POD-011', tone: 'progressing' }),
    pod('web', { status: 'ok', reason: 'Running', ruleId: 'TRI-POD-000' }),
    pod('done', { status: 'ok', reason: 'Completed', ruleId: 'TRI-POD-000', tone: 'neutral' }),
  ];

  it('buckets by verdict, not phase', () => {
    const b = bucketByHealth(rows);
    expect(b.all).toHaveLength(5);
    expect(b.failed.map((p) => p.name)).toEqual(['crash']);
    expect(b.warning.map((p) => p.name)).toEqual(['slow']);
    expect(b.progressing.map((p) => p.name)).toEqual(['new']);
    expect(b.healthy.map((p) => p.name)).toEqual(['web']);
    expect(b.completed.map((p) => p.name)).toEqual(['done']);
  });
});

describe('resourceState', () => {
  afterEach(() => vi.useRealTimers());

  it('moves loading -> ready and stamps lastSyncedAt', () => {
    const s = resourceState('test-a');
    expect(get(s).status).toBe('idle');
    markLoading('test-a');
    expect(get(s).status).toBe('loading');
    markSynced('test-a', 1234);
    expect(get(s)).toEqual({ status: 'ready', lastSyncedAt: 1234 });
    expect(get(lastSyncedAt)).toBe(1234);
  });

  it('tells forbidden from other errors', () => {
    expect(isForbiddenError('Kubernetes API error: pods is forbidden: User "x" cannot list')).toBe(true);
    expect(isForbiddenError('ApiError: 403 Forbidden')).toBe(true);
    expect(isForbiddenError('connection refused')).toBe(false);

    markFailed('test-b', 'pods is forbidden');
    expect(get(resourceState('test-b')).status).toBe('forbidden');
    markFailed('test-c', 'connection refused');
    expect(get(resourceState('test-c'))).toMatchObject({ status: 'error', error: 'connection refused' });
  });
});
