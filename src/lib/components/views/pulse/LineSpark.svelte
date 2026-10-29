<script lang="ts" module>
  import { HOUR_MS, type Sample } from '../../../pulse/history';

  /**
   * Polyline points for a sparkline (preview `lineSpark`), placed by time
   * across the window so a gap in sampling reads as a gap, not a squeeze.
   * `fixed` uses a 0-100 domain (percent metrics, with the dashed 90% line);
   * otherwise the domain hugs the data. Null samples are skipped.
   */
  export function linePoints(
    samples: readonly Sample[],
    now: number,
    w: number,
    h: number,
    fixed: boolean,
    windowMs = HOUR_MS
  ): string {
    const pts = samples.filter((s): s is { t: number; v: number } => s.v !== null && s.t >= now - windowMs && s.t <= now);
    if (pts.length < 2) return '';
    const vals = pts.map((p) => p.v);
    const mn = fixed ? 0 : Math.min(...vals) - 2;
    const mx = fixed ? 100 : Math.max(...vals) + 2;
    const from = now - windowMs;
    const x = (t: number) => ((t - from) / windowMs) * w;
    const y = (v: number) => h - 2 - ((v - mn) / (mx - mn || 1)) * (h - 4);
    return pts.map((p) => `${x(p.t).toFixed(1)},${y(p.v).toFixed(1)}`).join(' ');
  }
</script>

<script lang="ts">
  let {
    samples,
    now,
    w = 300,
    h = 44,
    fixed = false,
  }: { samples: readonly Sample[]; now: number; w?: number; h?: number; fixed?: boolean } = $props();

  const points = $derived(linePoints(samples, now, w, h, fixed));
  const last = $derived([...samples].reverse().find((s) => s.v !== null)?.v ?? null);
  const stroke = $derived(
    !fixed
      ? 'var(--ui-muted)'
      : last !== null && last >= 90
        ? 'var(--ui-danger-solid)'
        : last !== null && last >= 75
          ? 'var(--ui-warning-solid)'
          : 'var(--ui-accent)'
  );
  const y90 = $derived(h - 2 - 0.9 * (h - 4));
</script>

<svg class="ln" viewBox="0 0 {w} {h}" preserveAspectRatio="none" aria-hidden="true">
  {#if fixed}
    <line x1="0" x2={w} y1={y90} y2={y90} stroke="var(--ui-line-strong)" stroke-dasharray="3 3" vector-effect="non-scaling-stroke" />
  {/if}
  {#if points}
    <polyline {points} fill="none" {stroke} stroke-width="1.5" stroke-linejoin="round" vector-effect="non-scaling-stroke" />
  {/if}
</svg>
