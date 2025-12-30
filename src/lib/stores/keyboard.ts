import { writable, derived, get } from 'svelte/store';

// Selected row index for current view (-1 = no selection)
export const selectedRowIndex = writable<number>(-1);

// Whether keyboard navigation is active (user has started using j/k)
export const keyboardNavActive = writable<boolean>(false);

// Total rows in current view (set by each list component)
export const totalRows = writable<number>(0);

// Show keyboard shortcuts help modal
export const showKeyboardHelp = writable<boolean>(false);

// Move selection up
export function moveUp() {
  const total = get(totalRows);
  if (total === 0) return;

  keyboardNavActive.set(true);
  selectedRowIndex.update(i => {
    if (i <= 0) return total - 1; // Wrap to bottom
    return i - 1;
  });
}

// Move selection down
export function moveDown() {
  const total = get(totalRows);
  if (total === 0) return;

  keyboardNavActive.set(true);
  selectedRowIndex.update(i => {
    if (i >= total - 1) return 0; // Wrap to top
    return i + 1;
  });
}

// Go to first row
export function goToFirst() {
  const total = get(totalRows);
  if (total === 0) return;

  keyboardNavActive.set(true);
  selectedRowIndex.set(0);
}

// Go to last row
export function goToLast() {
  const total = get(totalRows);
  if (total === 0) return;

  keyboardNavActive.set(true);
  selectedRowIndex.set(total - 1);
}

// Clear selection
export function clearSelection() {
  selectedRowIndex.set(-1);
  keyboardNavActive.set(false);
}

// Reset when view changes
export function resetNavigation() {
  selectedRowIndex.set(-1);
  keyboardNavActive.set(false);
  totalRows.set(0);
}

// Keyboard shortcuts configuration
export interface KeyboardShortcut {
  key: string;
  description: string;
  category: 'navigation' | 'actions' | 'global';
}

export const keyboardShortcuts: KeyboardShortcut[] = [
  // Navigation
  { key: 'j / ↓', description: 'Move down', category: 'navigation' },
  { key: 'k / ↑', description: 'Move up', category: 'navigation' },
  { key: 'g g', description: 'Go to first row', category: 'navigation' },
  { key: 'G', description: 'Go to last row', category: 'navigation' },
  { key: 'Enter', description: 'Open detail', category: 'navigation' },
  { key: 'Esc', description: 'Clear selection / Close modal', category: 'navigation' },

  // Actions (context-dependent)
  { key: 'l', description: 'View logs (Pods)', category: 'actions' },
  { key: 'e', description: 'Exec into pod (Pods)', category: 'actions' },
  { key: 'd', description: 'Delete resource', category: 'actions' },
  { key: 'y', description: 'Copy resource name', category: 'actions' },
  { key: 'r', description: 'Refresh view', category: 'actions' },

  // Global
  { key: '⌘K / /', description: 'Open search', category: 'global' },
  { key: '⌘F', description: 'Focus filter input', category: 'global' },
  { key: '⌘P', description: 'Port forwards panel', category: 'global' },
  { key: '⌘W', description: 'Close detail window', category: 'global' },
  { key: '?', description: 'Show keyboard shortcuts', category: 'global' },
];
