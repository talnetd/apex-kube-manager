<script lang="ts" module>
  /**
   * Polyline points for a sparkline (preview `lineSpark`). `fixed` uses a
   * 0-100 domain (percent metrics, with the dashed 90% line); otherwise the
   * domain hugs the data. Nulls are skipped; gaps are rare at a 5s poll.
   */
  export function linePoints(values: readonly (number | null)[], w: number, h: number, fixed: boolean): string {
    const pts = values.map((v, i) => [i, v] as const).filter((p): p is readonly [number, number] => p[1] !== null);
    if (pts.length < 2) return '';
    const vals = pts.map((p) => p[1]);
    const mn = fixed ? 0 : Math.min(...vals) - 2;
    const mx = fixed ? 100 : Math.max(...vals) + 2;
    const n = values.length;
    const y = (v: number) => h - 2 - ((v - mn) / (mx - mn || 1)) * (h - 4);
    return pts.map(([i, v]) => `${((i * w) / (n - 1)).toFixed(1)},${y(v).toFixed(1)}`).join(' ');
  }
</script>

<script lang="ts">
  let {
    values,
    w = 300,
    h = 44,
    fixed = false,
  }: { values: readonly (number | null)[]; w?: number; h?: number; fixed?: boolean } = $props();

  const points = $derived(linePoints(values, w, h, fixed));
  const last = $derived([...values].reverse().find((v) => v !== null) ?? null);
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
