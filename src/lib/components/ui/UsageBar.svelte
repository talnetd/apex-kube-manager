<script lang="ts">
  // Compact usage bar for table cells. `percent` is null when the metric
  // source is unavailable (no metrics-server, no kubelet access).
  let {
    percent,
    detail = '',
    unavailableHint = 'No metrics available',
  }: {
    percent: number | null;
    detail?: string;
    unavailableHint?: string;
  } = $props();

  // The bar width clamps, but the label keeps the real number: usage can
  // exceed allocatable, since system-reserved overhead is counted too.
  const value = $derived(percent === null ? 0 : Math.max(0, percent));
  const width = $derived(Math.min(100, value));

  const barColor = $derived(
    value >= 90 ? 'bg-accent-error' : value >= 75 ? 'bg-accent-warning' : 'bg-accent-primary'
  );
  const textColor = $derived(
    value >= 90 ? 'text-accent-error' : value >= 75 ? 'text-accent-warning' : 'text-text-secondary'
  );
</script>

{#if percent === null}
  <span class="text-text-muted text-xs" title={unavailableHint}>&ndash;</span>
{:else}
  <div class="flex items-center gap-2" title={detail}>
    <div class="w-14 h-1.5 rounded-sm bg-bg-tertiary border border-border-subtle overflow-hidden shrink-0">
      <div class="h-full {barColor} transition-all duration-300" style="width:{width}%"></div>
    </div>
    <span class="text-xs tabular-nums {textColor} w-9 text-right">{Math.round(value)}%</span>
  </div>
{/if}
