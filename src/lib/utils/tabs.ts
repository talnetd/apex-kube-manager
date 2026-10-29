// src/lib/utils/tabs.ts
// Keyboard model for a tablist (WAI-ARIA tabs pattern, automatic
// activation): Left/Right wrap, Home/End jump. Pure, so it's unit-tested.

/** The tab index a key moves to, or null when the key isn't a tab key. */
export function nextTabIndex(current: number, key: string, count: number): number | null {
  if (count <= 0) return null;
  switch (key) {
    case 'ArrowRight':
      return (current + 1) % count;
    case 'ArrowLeft':
      return (current - 1 + count) % count;
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
    default:
      return null;
  }
}
