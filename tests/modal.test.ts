import { describe, it, expect, vi } from 'vitest';
import { modal } from '../src/lib/actions/modal';

function esc(target: HTMLElement, prevented = false) {
  const e = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true });
  if (prevented) e.preventDefault();
  target.dispatchEvent(e);
  return e;
}

describe('modal action', () => {
  it('focuses the dialog and closes on Escape', () => {
    const node = document.createElement('div');
    node.tabIndex = -1;
    document.body.append(node);
    const close = vi.fn();
    const a = modal(node, close);
    expect(document.activeElement).toBe(node);
    esc(node);
    expect(close).toHaveBeenCalledTimes(1);
    a.destroy();
    node.remove();
  });

  it('leaves an Escape an inner control already handled', () => {
    const node = document.createElement('div');
    const inner = document.createElement('button');
    node.append(inner);
    document.body.append(node);
    inner.addEventListener('keydown', (e) => e.preventDefault()); // e.g. open menu
    const close = vi.fn();
    const a = modal(node, close);
    esc(inner);
    expect(close).not.toHaveBeenCalled();
    a.destroy();
    node.remove();
  });
});
