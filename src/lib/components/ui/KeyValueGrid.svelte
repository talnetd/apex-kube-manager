<script lang="ts">
  // Labels / annotations as outlined mono `key=value` chips (Geist preview
  // `.lab`). Long annotation values wrap instead of widening the window.
  interface Props {
    data: Record<string, string>;
    emptyText?: string;
    /** CSS max-height, e.g. "12rem"; scrolls inside. */
    maxHeight?: string;
  }

  let { data, emptyText = 'None', maxHeight = '' }: Props = $props();

  const entries = $derived(Object.entries(data));
</script>

<div class="kv" style:max-height={maxHeight || undefined} class:scroll={!!maxHeight}>
  {#if entries.length > 0}
    {#each entries as [key, value]}
      <span class="lab" title={`${key}=${value}`}><span class="k">{key}</span>{#if value}<span class="eq">=</span><span class="v">{value}</span>{/if}</span>
    {/each}
  {:else}
    <span class="text-muted text-sm">{emptyText}</span>
  {/if}
</div>

<style>
  .kv {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .kv.scroll {
    overflow: auto;
  }
  .lab {
    max-width: 100%;
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 18px;
    padding: 0 6px;
    border: 1px solid var(--ui-line-strong);
    border-radius: 4px;
    color: var(--ui-secondary);
    overflow-wrap: anywhere;
  }
  .k {
    color: var(--ui-primary);
  }
  .eq {
    color: var(--ui-faint);
  }
</style>
