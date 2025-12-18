<script lang="ts">
  interface Props {
    value: string;
    placeholder?: string;
    onchange: (value: string) => void;
  }

  let { value = '', placeholder = 'Filter...', onchange }: Props = $props();

  let isExpanded = $state(false);
  let inputRef: HTMLInputElement;

  function toggle() {
    isExpanded = !isExpanded;
    if (isExpanded) {
      // Focus input when expanded
      setTimeout(() => inputRef?.focus(), 0);
    } else {
      // Clear filter when collapsed
      onchange('');
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isExpanded = false;
      onchange('');
    }
  }
</script>

<div class="relative flex items-center">
  {#if isExpanded}
    <div class="flex items-center gap-2 bg-bg-tertiary rounded-lg border border-border-subtle px-2 py-1">
      <svg class="w-3.5 h-3.5 text-text-muted flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <input
        bind:this={inputRef}
        type="text"
        {placeholder}
        class="w-40 bg-transparent text-sm text-text-primary placeholder-text-muted focus:outline-none"
        {value}
        oninput={(e) => onchange(e.currentTarget.value)}
        onkeydown={handleKeydown}
      />
      {#if value}
        <button
          type="button"
          class="text-text-muted hover:text-text-primary"
          onclick={() => onchange('')}
          title="Clear filter"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {:else}
        <button
          type="button"
          class="text-text-muted hover:text-text-primary"
          onclick={toggle}
          title="Close filter"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
    </div>
  {:else}
    <button
      type="button"
      class="p-1.5 rounded-lg hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
      onclick={toggle}
      title="Filter list"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
      </svg>
    </button>
  {/if}
</div>
