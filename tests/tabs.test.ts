import { describe, it, expect } from 'vitest';
import { nextTabIndex } from '../src/lib/utils/tabs';

describe('nextTabIndex', () => {
  it('moves and wraps with the arrow keys', () => {
    expect(nextTabIndex(0, 'ArrowRight', 4)).toBe(1);
    expect(nextTabIndex(3, 'ArrowRight', 4)).toBe(0);
    expect(nextTabIndex(0, 'ArrowLeft', 4)).toBe(3);
  });
  it('jumps with Home and End', () => {
    expect(nextTabIndex(2, 'Home', 4)).toBe(0);
    expect(nextTabIndex(1, 'End', 4)).toBe(3);
  });
  it('ignores other keys and empty lists', () => {
    expect(nextTabIndex(1, 'Enter', 4)).toBeNull();
    expect(nextTabIndex(0, 'ArrowRight', 0)).toBeNull();
  });
});
