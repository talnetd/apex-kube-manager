<script lang="ts">
  interface Props {
    label: string;
    field: string;
    sortField: string;
    sortDirection: 'asc' | 'desc';
    onSort: (field: string) => void;
    class?: string;
  }

  let { label, field, sortField, sortDirection, onSort, class: className = '' }: Props = $props();

  const isActive = $derived(sortField === field);
</script>

<th
  onclick={() => onSort(field)}
  class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium cursor-pointer hover:text-text-primary select-none {className}"
>
  <div class="flex items-center gap-1">
    <span>{label}</span>
    <span class="w-3 h-3 flex items-center justify-center">
      {#if isActive}
        <svg class="w-3 h-3 text-accent-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          {#if sortDirection === 'asc'}
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
          {:else}
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          {/if}
        </svg>
      {:else}
        <svg class="w-3 h-3 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
        </svg>
      {/if}
    </span>
  </div>
</th>
