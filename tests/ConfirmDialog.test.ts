import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { mount, unmount, flushSync, tick } from 'svelte';
import ConfirmDialog from '../src/lib/components/ui/ConfirmDialog.svelte';
import { settings, defaultSettings } from '../src/lib/stores/settings';
import { unmarkProd } from '../src/lib/stores/production';

type Props = Parameters<typeof mount<typeof ConfirmDialog>>[1]['props'];

let app: ReturnType<typeof mount> | null = null;

async function open(props: Partial<NonNullable<Props>> & { context: string }) {
  const onConfirm = vi.fn(async () => {});
  const { context, ...rest } = props;
  app = mount(ConfirmDialog, {
    target: document.body,
    props: {
      open: true,
      title: 'Restart deployment',
      resource: { kind: 'Deployment', name: 'checkout', namespace: 'payments', context },
      body: 'Pods are replaced one by one.',
      confirmLabel: 'Restart',
      onConfirm,
      ...rest,
    },
  });
  flushSync();
  await tick();
  await Promise.resolve();
  const dialog = document.querySelector<HTMLElement>('[role="dialog"], [role="alertdialog"]')!;
  const q = <E extends Element = HTMLElement>(id: string) => document.querySelector<E>(`[data-testid="${id}"]`);
  return { onConfirm, dialog, q };
}

function key(target: Element, k: string, init: KeyboardEventInit = {}) {
  target.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true, ...init }));
}

async function settle() {
  flushSync();
  await tick();
  await Promise.resolve();
}

beforeEach(() => {
  settings.set({ ...defaultSettings, prodOverrides: {}, mutedIssues: {} });
});

afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

describe('ConfirmDialog', () => {
  it('tier 1 off prod: no banner, focus on confirm, Enter confirms', async () => {
    const { onConfirm, dialog, q } = await open({ context: 'staging' });
    expect(q('prod-banner')).toBeNull();
    expect(dialog.getAttribute('role')).toBe('dialog');
    expect(document.activeElement).toBe(q('confirm'));
    expect(q('confirm')!.textContent).toContain('Restart');
    expect(q('confirm')!.textContent).not.toContain('on staging');

    // Enter from a non-button target inside the dialog confirms in tier 1.
    key(dialog, 'Enter');
    await settle();
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('auto-escalates on a context named eks-prod-eu', async () => {
    const { onConfirm, dialog, q } = await open({ context: 'eks-prod-eu' });
    expect(q('prod-banner')).not.toBeNull();
    expect(q('prod-banner')!.textContent).toContain('Production context: eks-prod-eu');
    expect(dialog.getAttribute('role')).toBe('alertdialog');
    expect(q('confirm')!.textContent).toContain('Restart on eks-prod-eu');
    // Focus starts on Cancel; plain Enter on the dialog does not confirm.
    expect(document.activeElement).toBe(q('cancel'));
    key(dialog, 'Enter');
    await settle();
    expect(onConfirm).not.toHaveBeenCalled();
    // Cmd/Ctrl+Enter does.
    key(dialog, 'Enter', { ctrlKey: true });
    await settle();
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('does not escalate a prod-looking context that was unmarked', async () => {
    unmarkProd('eks-prod-eu');
    const { onConfirm, dialog, q } = await open({ context: 'eks-prod-eu' });
    expect(q('prod-banner')).toBeNull();
    expect(q('confirm')!.textContent).not.toContain('on eks-prod-eu');
    expect(document.activeElement).toBe(q('confirm'));
    key(dialog, 'Enter');
    await settle();
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('tier 2 on prod requires typing the name before confirming', async () => {
    const { onConfirm, dialog, q } = await open({ context: 'eks-prod-eu', tone: 'danger', confirmLabel: 'Delete pod' });
    const input = q<HTMLInputElement>('confirm-input')!;
    const confirm = q<HTMLButtonElement>('confirm')!;
    expect(input).not.toBeNull();
    expect(document.activeElement).toBe(input);
    expect(confirm.disabled).toBe(true);

    key(dialog, 'Enter', { metaKey: true });
    await settle();
    expect(onConfirm).not.toHaveBeenCalled();

    input.value = 'checkout';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    await settle();
    expect(confirm.disabled).toBe(false);
    confirm.click();
    await settle();
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('Escape cancels and keys never reach window listeners', async () => {
    const onClose = vi.fn();
    const windowSpy = vi.fn();
    window.addEventListener('keydown', windowSpy);
    const { dialog } = await open({ context: 'staging', onClose });
    key(dialog, 'j');
    key(dialog, 'Escape');
    await settle();
    expect(windowSpy).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledTimes(1);
    window.removeEventListener('keydown', windowSpy);
  });

  it('keeps the dialog open and shows the error when onConfirm throws', async () => {
    const failing = vi.fn(async () => {
      throw new Error('forbidden: deployments.apps "checkout"');
    });
    const { dialog } = await open({ context: 'staging', onConfirm: failing });
    key(dialog, 'Enter');
    await settle();
    await settle();
    expect(failing).toHaveBeenCalled();
    expect(document.querySelector('[role="alert"]')?.textContent).toContain('forbidden');
    expect(document.querySelector('[role="dialog"]')).not.toBeNull();
  });

  it('after a rejected confirm, focus stays inside and Esc closes', async () => {
    const onClose = vi.fn();
    const failing = vi.fn(async () => {
      throw new Error('boom');
    });
    const { dialog, q } = await open({ context: 'staging', onConfirm: failing, onClose });
    key(dialog, 'Enter');
    await settle();
    await settle();
    expect(failing).toHaveBeenCalled();
    expect(dialog.contains(document.activeElement)).toBe(true);
    expect(document.activeElement).toBe(q('cancel'));

    key(document.activeElement!, 'Escape');
    await settle();
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('keys from outside the dialog (focus lost to body) are swallowed; Esc still closes', async () => {
    const onClose = vi.fn();
    const windowSpy = vi.fn();
    window.addEventListener('keydown', windowSpy);
    const { onConfirm } = await open({ context: 'staging', onClose });
    (document.activeElement as HTMLElement | null)?.blur();
    key(document.body, 'Enter');
    key(document.body, 'd');
    await settle();
    expect(windowSpy).not.toHaveBeenCalled();
    expect(onConfirm).not.toHaveBeenCalled();
    key(document.body, 'Escape');
    await settle();
    expect(onClose).toHaveBeenCalledTimes(1);
    window.removeEventListener('keydown', windowSpy);
  });
});
