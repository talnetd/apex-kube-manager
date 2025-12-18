<script lang="ts">
  interface Props {
    data: Record<string, string>;
    emptyText?: string;
    maxHeight?: string;
  }

  let { data, emptyText = 'None', maxHeight = '' }: Props = $props();

  const entries = $derived(Object.entries(data));
</script>

<div class="bg-bg-secondary rounded-lg p-4 {maxHeight ? `max-h-[${maxHeight}] overflow-auto` : ''}">
  {#if entries.length > 0}
    <div class="space-y-2">
      {#each entries as [key, value]}
        <div class="flex flex-wrap gap-2">
          <span class="px-2 py-0.5 bg-accent-primary/20 text-accent-primary text-xs font-mono rounded">
            {key}
          </span>
          <span class="text-text-secondary text-sm font-mono break-all">
            {value}
          </span>
        </div>
      {/each}
    </div>
  {:else}
    <span class="text-text-muted text-sm">{emptyText}</span>
  {/if}
</div>
