import { describe, it, expect, vi, afterEach } from 'vitest';
import { mount, unmount, flushSync, tick } from 'svelte';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => null) }));

import SecretCreator from '../src/lib/components/ui/SecretCreator.svelte';
import { currentContext } from '../src/lib/stores/kubernetes';

let app: ReturnType<typeof mount> | null = null;
afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

async function open() {
  currentContext.set('ctx');
  const onclose = vi.fn();
  app = mount(SecretCreator, { target: document.body, props: { show: true, onclose, context: 'ctx' } });
  flushSync();
  await tick();
  const dialog = document.querySelector<HTMLElement>('[aria-labelledby="sc-title"]')!;
  return { onclose, dialog };
}
const escape = (el: HTMLElement) =>
  el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
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
    input.value = 'db-pass';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();

    escape(input);
    flushSync();
    await tick();
    expect(onclose).not.toHaveBeenCalled();
    expect(discardPrompt()).toBeDefined();
    expect(input.value).toBe('db-pass');

    const cancel = Array.from(dialog.querySelectorAll('button')).find((b) => b.textContent?.trim() === 'Cancel')!;
    expect(cancel).toBeDefined();
  });
});
