<script lang="ts" module>
  export interface SparkSegment {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    level: 'normal' | 'warning' | 'failed';
  }

  export const SPARK_W = 120;
  export const SPARK_H = 24;
  const PAD = 2;

  export function levelOf(v: number, warnAt = 75, failAt = 90): SparkSegment['level'] {
    return v >= failAt ? 'failed' : v >= warnAt ? 'warning' : 'normal';
  }

  /** Fixed 0-100 y-domain (rows stay comparable); nulls break the line, never interpolated. */
  export function sparkSegments(values: readonly (number | null)[], warnAt = 75, failAt = 90): SparkSegment[] {
    const n = values.length;
    if (n < 2) return [];
    const x = (i: number) => PAD + (i * (SPARK_W - PAD * 2)) / (n - 1);
    const y = (v: number) => PAD + (1 - Math.max(0, Math.min(100, v)) / 100) * (SPARK_H - PAD * 2);
    const out: SparkSegment[] = [];
    for (let i = 1; i < n; i++) {
      const a = values[i - 1];
      const b = values[i];
      if (a == null || b == null) continue;
      out.push({ x1: x(i - 1), y1: y(a), x2: x(i), y2: y(b), level: levelOf(Math.max(a, b), warnAt, failAt) });
    }
    return out;
  }
</script>

<script lang="ts">
  // Sparkline (spec 6.6): 120×24 comfortable / 96×18 compact (via CSS),
  // 1.5px line in text-muted, segments above 75% warning-solid and above
  // 90% danger-solid, dashed guide at 90%, 3px dot on the last point.
  // No animation.
  let {
    values,
    label,
    times,
    format = (v: number) => `${Math.round(v)}%`,
    warnAt = 75,
    failAt = 90,
  }: {
    /** Percent values (0-100); null = gap. */
    values: (number | null)[];
    /** Accessible name, e.g. "CPU 62 percent, up 8 points in the last 15 minutes". */
    label: string;
    /** Optional timestamps (ms) for the hover tooltip. */
    times?: number[];
    format?: (v: number) => string;
    warnAt?: number;
    failAt?: number;
  } = $props();

  const segments = $derived(sparkSegments(values, warnAt, failAt));
  const n = $derived(values.length);
  const xAt = (i: number) => (n <= 1 ? SPARK_W / 2 : PAD + (i * (SPARK_W - PAD * 2)) / (n - 1));
  const yAt = (v: number) => PAD + (1 - Math.max(0, Math.min(100, v)) / 100) * (SPARK_H - PAD * 2);
  const lastIdx = $derived.by(() => {
    for (let i = values.length - 1; i >= 0; i--) if (values[i] != null) return i;
    return -1;
  });
  const guideY = $derived(yAt(failAt));

  let hover = $state<number | null>(null);
  let svgEl = $state<SVGSVGElement>();

  function onMove(e: MouseEvent) {
    if (!svgEl || n === 0) return;
    const r = svgEl.getBoundingClientRect();
    const rel = (e.clientX - r.left) / r.width;
    const i = Math.round(rel * (n - 1));
    hover = Math.max(0, Math.min(n - 1, i));
  }

  const hoverText = $derived.by(() => {
    if (hover === null) return '';
    const v = values[hover];
    const val = v == null ? 'no data' : format(v);
    const t = times?.[hover];
    return t ? `${val} · ${new Date(t).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}` : val;
  });

  const COLOR: Record<SparkSegment['level'], string> = {
    normal: 'var(--ui-muted)',
    warning: 'var(--ui-warning-solid)',
    failed: 'var(--ui-danger-solid)',
  };
</script>

<span class="spark relative inline-block align-middle">
  <svg
    bind:this={svgEl}
    viewBox="0 0 {SPARK_W} {SPARK_H}"
    preserveAspectRatio="none"
    class="block w-full h-full overflow-visible"
    role="img"
    aria-label={label}
    onmousemove={onMove}
    onmouseleave={() => (hover = null)}
  >
    <line
      x1="0"
      x2={SPARK_W}
      y1={guideY}
      y2={guideY}
      stroke="var(--ui-line)"
      stroke-width="1"
      stroke-dasharray="2 2"
      vector-effect="non-scaling-stroke"
    />
    {#each segments as s}
      <line
        x1={s.x1}
        y1={s.y1}
        x2={s.x2}
        y2={s.y2}
        stroke={COLOR[s.level]}
        stroke-width="1.5"
        stroke-linecap="round"
        vector-effect="non-scaling-stroke"
      />
    {/each}
    {#if lastIdx >= 0}
      {@const v = values[lastIdx] as number}
      <circle cx={xAt(lastIdx)} cy={yAt(v)} r="1.5" fill={COLOR[levelOf(v, warnAt, failAt)]} />
    {/if}
    {#if hover !== null}
      <line
        x1={xAt(hover)}
        x2={xAt(hover)}
        y1="0"
        y2={SPARK_H}
        stroke="var(--ui-line-strong)"
        stroke-width="1"
        vector-effect="non-scaling-stroke"
      />
    {/if}
  </svg>
  {#if hover !== null}
    <span
      class="absolute left-1/2 -translate-x-1/2 bottom-full mb-1 px-1.5 py-0.5 rounded-md bg-surface shadow-popover text-2xs text-primary whitespace-nowrap tabular-nums pointer-events-none"
      aria-hidden="true">{hoverText}</span
    >
  {/if}
</span>

<style>
  .spark {
    width: 120px;
    height: 24px;
  }
  :global([data-density='compact']) .spark {
    width: 96px;
    height: 18px;
  }
</style>
