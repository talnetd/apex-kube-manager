import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { parseStoredTheme } from '../src/lib/stores/theme';
import { settings, parseSettings, defaultSettings } from '../src/lib/stores/settings';
import { density, toggleDensity } from '../src/lib/stores/density';
import {
  resolveProd,
  isProdContext,
  markProd,
  unmarkProd,
  resetProd,
  prodSource,
} from '../src/lib/stores/production';
import { toDisplay, worstHealth } from '../src/lib/utils/health';

beforeEach(() => {
  settings.set({ ...defaultSettings, prodOverrides: {}, mutedIssues: {} });
});

describe('theme', () => {
  it('migrates the retired neon theme to dark', () => {
    expect(parseStoredTheme('neon')).toBe('dark');
  });
  it('keeps valid values and defaults to system', () => {
    expect(parseStoredTheme('light')).toBe('light');
    expect(parseStoredTheme('system')).toBe('system');
    expect(parseStoredTheme(null)).toBe('system');
  });
});

describe('settings', () => {
  it('fills defaults and drops malformed fields', () => {
    const s = parseSettings(JSON.stringify({ density: 'huge', prodOverrides: { a: true, b: 'yes' } }));
    expect(s.density).toBe('comfortable');
    expect(s.prodOverrides).toEqual({ a: true });
    expect(s.notificationsEnabled).toBe(true);
    expect(parseSettings('not json')).toEqual(defaultSettings);
  });
});

describe('density', () => {
  it('defaults to comfortable, toggles, and sets data-density on <html>', () => {
    expect(get(density)).toBe('comfortable');
    expect(document.documentElement.dataset.density).toBe('comfortable');
    expect(toggleDensity()).toBe('compact');
    expect(document.documentElement.dataset.density).toBe('compact');
    expect(JSON.parse(localStorage.getItem('apex-kube-manager-settings')!).density).toBe('compact');
  });
});

describe('production guard (spec 4.6)', () => {
  it('auto-flags names matching /prod|prd/i', () => {
    expect(resolveProd('eks-prod-eu', {})).toBe(true);
    expect(resolveProd('gke-PRD-1', {})).toBe(true);
    expect(resolveProd('staging', {})).toBe(false);
    expect(resolveProd('', {})).toBe(false);
  });

  it('manual overrides win and reset returns to auto', () => {
    unmarkProd('products-dev');
    expect(isProdContext('products-dev')).toBe(false);
    expect(prodSource('products-dev')).toBe('manual');
    markProd('staging');
    expect(isProdContext('staging')).toBe(true);
    resetProd('staging');
    expect(isProdContext('staging')).toBe(false);
    expect(prodSource('eks-prod-eu')).toBe('auto');
    expect(prodSource('staging')).toBe('none');
  });
});

describe('health display mapping (spec 11.1)', () => {
  it('maps verdicts to display states and ranks', () => {
    expect(toDisplay({ status: 'fail', reason: 'CrashLoopBackOff', ruleId: 'TRI-POD-1' })).toMatchObject({
      health: 'failed',
      label: 'CrashLoopBackOff',
      rank: 0,
    });
    expect(toDisplay({ status: 'warn', reason: '1/3 ready', ruleId: 'x' }).health).toBe('warning');
    expect(toDisplay({ status: 'ok', reason: 'Rolling out', ruleId: 'x', tone: 'progressing' }).health).toBe(
      'progressing'
    );
    expect(toDisplay({ status: 'ok', reason: 'Completed', ruleId: 'x', tone: 'neutral' }).rank).toBe(5);
    expect(toDisplay({ status: 'unknown', reason: '', ruleId: 'x' })).toMatchObject({ health: 'unknown', label: 'Unknown' });
    expect(toDisplay({ status: 'ok', reason: 'Running', ruleId: 'x' }).health).toBe('healthy');
  });

  it('worst of several states wins', () => {
    expect(worstHealth(['healthy', 'warning', 'progressing'])).toBe('warning');
    expect(worstHealth([])).toBeNull();
  });
});
