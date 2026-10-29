// Dialog basics, in two parts:
// - `use:modal`: focus the dialog when it opens (so keys reach it, not the
//   list behind) and hand focus back to whatever opened it on close.
// - `onkeydown={escapeKeydown(close)}`: close on Escape. It must be a Svelte
//   `onkeydown` attribute, not a listener added here: Svelte delegates
//   keydown to the app root and runs handlers innermost first, so an inner
//   control (an open CustomSelect, a nested ConfirmDialog) gets to mark the
//   Escape handled before the dialog sees it. One press closes one thing.
export function modal(node: HTMLElement) {
  const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  if (!node.contains(document.activeElement)) node.focus({ preventScroll: true });
  return {
    destroy() {
      if (opener?.isConnected && !node.contains(opener)) opener.focus({ preventScroll: true });
    },
  };
}

export function escapeKeydown(close: () => void) {
  return (e: KeyboardEvent) => {
    // Escape during IME composition belongs to the input method.
    if (e.key !== 'Escape' || e.defaultPrevented || e.isComposing) return;
    e.preventDefault();
    e.stopPropagation(); // don't also clear the list selection behind
    close();
  };
}
