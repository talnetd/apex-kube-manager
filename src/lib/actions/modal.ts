// Dialog keyboard basics: focus the dialog when it opens (so keys reach it,
// not the list behind) and close it on Escape. An Escape something inside
// already handled (an open CustomSelect menu, a nested ConfirmDialog) is
// left alone, so one press closes one thing.
export function modal(node: HTMLElement, onEscape: () => void) {
  let close = onEscape;
  if (!node.contains(document.activeElement)) node.focus({ preventScroll: true });
  const onKeydown = (e: KeyboardEvent) => {
    if (e.key !== 'Escape' || e.defaultPrevented) return;
    e.preventDefault();
    e.stopPropagation(); // don't also clear the list selection behind
    close();
  };
  node.addEventListener('keydown', onKeydown);
  return {
    update(next: () => void) {
      close = next;
    },
    destroy() {
      node.removeEventListener('keydown', onKeydown);
    },
  };
}
