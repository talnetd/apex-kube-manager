import { describe, it, expect, vi } from 'vitest';
import { modal, escapeKeydown } from '../src/lib/actions/modal';

const key = (init: KeyboardEventInit) => new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });

describe('escapeKeydown', () => {
  it('closes on Escape and stops it reaching the list behind', () => {
    const close = vi.fn();
    const e = key({ key: 'Escape' });
    const stop = vi.spyOn(e, 'stopPropagation');
    escapeKeydown(close)(e);
    expect(close).toHaveBeenCalledTimes(1);
    expect(stop).toHaveBeenCalled();
  });

  it('leaves an Escape something inside already handled', () => {
    const close = vi.fn();
    const e = key({ key: 'Escape' });
    e.preventDefault();
    escapeKeydown(close)(e);
    expect(close).not.toHaveBeenCalled();
  });

  it('ignores Escape during IME composition and other keys', () => {
    const close = vi.fn();
    escapeKeydown(close)(key({ key: 'Escape', isComposing: true }));
    escapeKeydown(close)(key({ key: 'Enter' }));
    escapeKeydown(close)(key({ key: 'Escape', keyCode: 229 }));
    expect(close).not.toHaveBeenCalled();
  });
});

describe('modal action', () => {
  it('focuses the dialog, then returns focus to the opener', () => {
    const opener = document.createElement('button');
    const node = document.createElement('div');
    node.tabIndex = -1;
    document.body.append(opener, node);
    opener.focus();
    const a = modal(node);
    expect(document.activeElement).toBe(node);
    a.destroy();
    expect(document.activeElement).toBe(opener);
    opener.remove();
    node.remove();
  });
});
