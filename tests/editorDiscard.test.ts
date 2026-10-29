import { describe, it, expect, vi, afterEach } from 'vitest';
import { mount, unmount, flushSync, tick } from 'svelte';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => null) }));

import SecretCreator from '../src/lib/components/ui/SecretCreator.svelte';
import { currentContext, namespaces } from '../src/lib/stores/kubernetes';

let app: ReturnType<typeof mount> | null = null;
afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

type EditData = { name: string; namespace: string; secret_type: string; data: Record<string, string> };
async function open(editData: EditData | null = null) {
  currentContext.set('ctx');
  const onclose = vi.fn();
  app = mount(SecretCreator, { target: document.body, props: { show: true, onclose, context: 'ctx', editData } });
  flushSync();
  await tick();
  const dialog = document.querySelector<HTMLElement>('[aria-labelledby="sc-title"]')!;
  return { onclose, dialog };
}
const escape = (el: HTMLElement) =>
  el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
function type(input: HTMLInputElement, value: string) {
  input.value = value;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}
const discardPrompt = () => Array.from(document.querySelectorAll('h2, h3')).find((h) => h.textContent?.includes('Discard changes?'));

describe('SecretCreator close rule', () => {
  it('Escape on a clean form just closes', async () => {
    const { onclose, dialog } = await open();
    escape(dialog);
    flushSync();
    expect(onclose).toHaveBeenCalledTimes(1);
    expect(discardPrompt()).toBeUndefined();
  });

  it('Escape and Cancel with unsaved edits ask "Discard changes?" first', async () => {
    const { onclose, dialog } = await open();
    const input = document.querySelector<HTMLInputElement>('#sc-name')!;
    type(input, 'db-pass');

    escape(input);
    flushSync();
    await tick();
    expect(onclose).not.toHaveBeenCalled();
    expect(discardPrompt()).toBeDefined();
    expect(input.value).toBe('db-pass');

  });

  it('Cancel with unsaved edits asks too', async () => {
    const { onclose, dialog } = await open();
    type(document.querySelector<HTMLInputElement>('#sc-name')!, 'db-pass');
    const cancel = Array.from(dialog.querySelectorAll('button')).find((b) => b.textContent?.trim() === 'Cancel')!;
    cancel.click();
    flushSync();
    await tick();
    expect(onclose).not.toHaveBeenCalled();
    expect(discardPrompt()).toBeDefined();
  });

  it('a context switch that closes the editor clears a pending prompt', async () => {
    const { dialog } = await open();
    type(document.querySelector<HTMLInputElement>('#sc-name')!, 'db-pass');
    escape(dialog);
    flushSync();
    await tick();
    expect(discardPrompt()).toBeDefined();
    currentContext.set('other');
    flushSync();
    await tick();
    expect(discardPrompt()).toBeUndefined();
  });

  it('edit mode: a namespace list reload does not make edits look clean', async () => {
    const { onclose } = await open({ name: 'db', namespace: 'prod', secret_type: 'Opaque', data: { password: 'x' } });
    const key = document.querySelector<HTMLInputElement>('input[aria-label="Key 1"]')!;
    type(key, 'pw');
    namespaces.set(['default', 'prod', 'new-ns']);
    flushSync();
    await tick();
    expect(key.value).toBe('pw');
    escape(key);
    flushSync();
    await tick();
    expect(onclose).not.toHaveBeenCalled();
    expect(discardPrompt()).toBeDefined();
  });
});
