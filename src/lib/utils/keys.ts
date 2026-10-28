// src/lib/utils/keys.ts

/**
 * True when keyboard shortcuts must not fire because the user is typing
 * (spec 8): inputs, textareas, selects, contenteditable and comboboxes.
 */
export function isTypingTarget(el: Element | null | undefined): boolean {
  if (!el) return false;
  const tag = el.tagName.toLowerCase();
  if (tag === 'input' || tag === 'textarea' || tag === 'select') return true;
  const h = el as HTMLElement;
  if (h.isContentEditable) return true;
  const role = h.getAttribute?.('role');
  return role === 'combobox' || role === 'textbox';
}
