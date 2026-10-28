<script lang="ts">
  // Compact usage bar for table cells (spec 5.7). `percent` is null when the
  // metric source is unavailable (no metrics-server, no kubelet access); it
  // renders `–`, never a confident 0%.
  // Fill is neutral below 75%: usage isn't "good", it's just usage. Thresholds
  // (11.4) add a status icon after the number, so colour is never alone.
  import StatusIcon from './StatusIcon.svelte';

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
  const level = $derived(value >= 90 ? 'failed' : value >= 75 ? 'warning' : null);

  const barColor = $derived(
    level === 'failed' ? 'bg-danger-solid' : level === 'warning' ? 'bg-warning-solid' : 'bg-muted'
  );
  const textColor = $derived(
    level === 'failed' ? 'text-danger' : level === 'warning' ? 'text-warning' : 'text-secondary'
  );
</script>

{#if percent === null}
  <span class="text-faint text-xs" title={unavailableHint} aria-label={unavailableHint}>&ndash;</span>
{:else}
  <div class="flex items-center gap-2" title={detail}>
    <div class="w-14 h-1 rounded-sm bg-hover overflow-hidden shrink-0" aria-hidden="true">
      <div class="h-full rounded-sm {barColor} transition-[width] duration-150 ease-out" style="width:{width}%"></div>
    </div>
    <span class="inline-flex items-center gap-1 text-xs tabular-nums {textColor}">
      <span class="w-8 text-right">{Math.round(value)}%</span>
      {#if level}
        <StatusIcon health={level} colored={false} label={level === 'failed' ? 'at or above 90 percent' : 'at or above 75 percent'} />
      {:else}
        <span class="w-3" aria-hidden="true"></span>
      {/if}
    </span>
  </div>
{/if}
