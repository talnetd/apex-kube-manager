import { describe, it, expect, vi, afterEach } from 'vitest';
import { mount, unmount, flushSync, tick } from 'svelte';
import CustomSelect from '../src/lib/components/ui/CustomSelect.svelte';
import { escapeKeydown } from '../src/lib/actions/modal';

let app: ReturnType<typeof mount> | null = null;
afterEach(() => {
  if (app) unmount(app);
  app = null;
  document.body.innerHTML = '';
});

describe('CustomSelect inside a modal', () => {
  it('Escape after a click-open closes the menu, not the dialog', async () => {
    const dialog = document.createElement('div');
    dialog.tabIndex = -1;
    document.body.append(dialog);
    const close = vi.fn();
    // Svelte delegates keydown to the mount root (here `dialog`), so a
    // listener on its parent sees the event after the select's handler,
    // as the dialog's own onkeydown does in the app.
    const onKey = escapeKeydown(close);
    document.body.addEventListener('keydown', onKey);
    app = mount(CustomSelect, {
      target: dialog,
      props: { value: 'a', options: [{ value: 'a', label: 'A' }, { value: 'b', label: 'B' }], ariaLabel: 'Type' },
    });
    flushSync();
    const trigger = dialog.querySelector<HTMLButtonElement>('button[aria-label="Type"]')!;
    trigger.focus();
    trigger.click();
    flushSync();
    await tick();
    expect(dialog.querySelectorAll('button').length).toBeGreaterThan(1); // menu open

    trigger.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
    flushSync();
    expect(close).not.toHaveBeenCalled();
    expect(dialog.querySelectorAll('button').length).toBe(1); // menu closed

    trigger.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
    expect(close).toHaveBeenCalledTimes(1); // second press closes the dialog
    document.body.removeEventListener('keydown', onKey);
  });
});
