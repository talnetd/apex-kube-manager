// src/lib/components/ui/status-icons.ts
// The six status shapes (spec 11.1). Shared contract with mobile: the shape is
// what carries meaning, colour is redundant. Drawn on a 16-unit grid with
// stroke 2 and round caps; rendered at 12px (StatusIcon.svelte).
import type { Health } from '../../utils/health';

export interface IconPath {
  d: string;
  /** Filled instead of stroked (the progressing half-disc). */
  fill?: boolean;
}

export const STATUS_ICON_VIEWBOX = '0 0 16 16';

const CIRCLE = 'M14 8A6 6 0 1 1 2 8a6 6 0 0 1 12 0z';

export const STATUS_ICONS: Record<Health, IconPath[]> = {
  // ✕ inside an octagon
  failed: [
    { d: 'M5.5 1.75h5l3.75 3.75v5l-3.75 3.75h-5L1.75 10.5v-5z' },
    { d: 'M6 6l4 4M10 6l-4 4' },
  ],
  // ! inside a triangle
  warning: [
    { d: 'M8 1.9l6.4 11.35H1.6z' },
    { d: 'M8 6.25v2.75M8 11.25v.01' },
  ],
  // half-filled circle, static
  progressing: [{ d: CIRCLE }, { d: 'M8 2a6 6 0 0 0 0 12z', fill: true }],
  // ? inside a circle
  unknown: [
    { d: CIRCLE },
    { d: 'M6.3 6.3a1.75 1.75 0 1 1 2.45 1.6c-.45.22-.75.55-.75 1.05M8 11.1v.01' },
  ],
  // ✓ inside a circle
  healthy: [{ d: CIRCLE }, { d: 'M5.4 8.2l1.8 1.8 3.4-3.6' }],
  // empty circle
  neutral: [{ d: CIRCLE }],
};
