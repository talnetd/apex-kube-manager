import { describe, it, expect } from 'vitest';
import { resolveConfirm, typedMatches, baseTierOf } from '../src/lib/utils/confirm';

const res = (context: string) => ({ kind: 'Deployment', name: 'checkout', namespace: 'payments', context });

describe('baseTierOf', () => {
  it('derives the tier from tone and requireTyping', () => {
    expect(baseTierOf({})).toBe(1);
    expect(baseTierOf({ tone: 'danger' })).toBe(2);
    expect(baseTierOf({ tone: 'danger', requireTyping: 'ns' })).toBe(3);
    expect(baseTierOf({ tier: 2 })).toBe(2);
  });
});

describe('resolveConfirm (spec 7.4)', () => {
  it('tier 1 off prod: Enter confirms, focus on confirm, plain label', () => {
    const b = resolveConfirm({ resource: res('staging'), confirmLabel: 'Restart', isProd: false });
    expect(b.effectiveTier).toBe(1);
    expect(b.enterConfirms).toBe(true);
    expect(b.initialFocus).toBe('confirm');
    expect(b.confirmLabel).toBe('Restart');
    expect(b.danger).toBe(false);
    expect(b.role).toBe('dialog');
    expect(b.typeToConfirm).toBeNull();
  });

  it('tier 1 on prod behaves as tier 2 but keeps the default button style', () => {
    const b = resolveConfirm({ resource: res('eks-prod-eu'), confirmLabel: 'Restart', isProd: true });
    expect(b.effectiveTier).toBe(2);
    expect(b.enterConfirms).toBe(false);
    expect(b.initialFocus).toBe('cancel');
    expect(b.danger).toBe(false);
    expect(b.prod).toBe(true);
    expect(b.confirmLabel).toBe('Restart on eks-prod-eu');
    expect(b.role).toBe('alertdialog');
  });

  it('tier 2 off prod: danger, focus on Cancel, Enter alone does not confirm', () => {
    const b = resolveConfirm({ resource: res('dev'), confirmLabel: 'Delete', tone: 'danger', isProd: false });
    expect(b.effectiveTier).toBe(2);
    expect(b.danger).toBe(true);
    expect(b.initialFocus).toBe('cancel');
    expect(b.enterConfirms).toBe(false);
    expect(b.typeToConfirm).toBeNull();
  });

  it('tier 2 on prod becomes type-to-confirm with the resource name', () => {
    const b = resolveConfirm({ resource: res('eks-prod-eu'), confirmLabel: 'Delete', tone: 'danger', isProd: true });
    expect(b.effectiveTier).toBe(3);
    expect(b.typeToConfirm).toBe('checkout');
    expect(b.initialFocus).toBe('input');
  });

  it('tier 3 on prod requires context/name', () => {
    const b = resolveConfirm({
      resource: res('eks-prod-eu'),
      confirmLabel: 'Delete',
      tone: 'danger',
      requireTyping: 'payments',
      isProd: true,
    });
    expect(b.effectiveTier).toBe(4);
    expect(b.typeToConfirm).toBe('eks-prod-eu/payments');
  });

  it('prodEscalation: false never escalates (the unmark-production confirm)', () => {
    const b = resolveConfirm({ resource: res('eks-prod-eu'), confirmLabel: 'Unmark', isProd: true, prodEscalation: false });
    expect(b.effectiveTier).toBe(1);
    expect(b.prod).toBe(false);
    expect(b.confirmLabel).toBe('Unmark');
  });

  it('typedMatches is exact after trimming', () => {
    const b = { typeToConfirm: 'checkout' };
    expect(typedMatches(b, 'checkou')).toBe(false);
    expect(typedMatches(b, ' checkout ')).toBe(true);
    expect(typedMatches({ typeToConfirm: null }, '')).toBe(true);
  });
});
