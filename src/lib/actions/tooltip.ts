// src/lib/actions/tooltip.ts
// `use:tooltip={'Logs · L'}` (spec 5.7): 400ms delay, bg-surface,
// shadow-popover, text-xs, max-width 280px. Tooltips are never the only
// label: icon-only buttons still need aria-label.

export const TOOLTIP_DELAY_MS = 400;

export function tooltip(node: HTMLElement, text: string | null | undefined) {
  let current = text ?? '';
  let timer: ReturnType<typeof setTimeout> | null = null;
  let tip: HTMLDivElement | null = null;

  function place() {
    if (!tip) return;
    const r = node.getBoundingClientRect();
    const t = tip.getBoundingClientRect();
    const margin = 6;
    let top = r.bottom + margin;
    if (top + t.height > window.innerHeight - 4) top = r.top - t.height - margin;
    let left = r.left + r.width / 2 - t.width / 2;
    left = Math.max(4, Math.min(left, window.innerWidth - t.width - 4));
    tip.style.top = `${Math.round(top)}px`;
    tip.style.left = `${Math.round(left)}px`;
  }

  function show() {
    if (!current || tip) return;
    tip = document.createElement('div');
    tip.setAttribute('role', 'tooltip');
    tip.className =
      'fixed z-[1000] pointer-events-none bg-surface text-primary shadow-popover rounded-md px-2 py-1 text-xs max-w-[280px]';
    tip.textContent = current;
    document.body.appendChild(tip);
    place();
  }

  function hide() {
    if (timer) clearTimeout(timer);
    timer = null;
    tip?.remove();
    tip = null;
  }

  function schedule() {
    if (timer || tip) return;
    timer = setTimeout(() => {
      timer = null;
      show();
    }, TOOLTIP_DELAY_MS);
  }

  node.addEventListener('mouseenter', schedule);
  node.addEventListener('mouseleave', hide);
  node.addEventListener('focusin', schedule);
  node.addEventListener('focusout', hide);
  node.addEventListener('pointerdown', hide);

  return {
    update(next: string | null | undefined) {
      current = next ?? '';
      if (tip) {
        if (current) {
          tip.textContent = current;
          place();
        } else hide();
      }
    },
    destroy() {
      hide();
      node.removeEventListener('mouseenter', schedule);
      node.removeEventListener('mouseleave', hide);
      node.removeEventListener('focusin', schedule);
      node.removeEventListener('focusout', hide);
      node.removeEventListener('pointerdown', hide);
    },
  };
}
