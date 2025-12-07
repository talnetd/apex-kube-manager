<script lang="ts">
  interface Tab {
    id: string;
    label: string;
    count?: number;
  }

  interface Props {
    tabs: Tab[];
    activeTab: string;
    onTabChange: (id: string) => void;
  }

  let { tabs, activeTab, onTabChange }: Props = $props();
</script>

<div class="flex items-center gap-1 border-b border-border-subtle">
  {#each tabs as tab}
    <button
      onclick={() => onTabChange(tab.id)}
      class="px-4 py-2 text-sm transition-colors relative
        {activeTab === tab.id
          ? 'text-accent-primary'
          : 'text-text-secondary hover:text-text-primary'}"
    >
      <span>{tab.label}</span>
      {#if tab.count !== undefined}
        <span class="ml-1.5 text-text-muted">({tab.count})</span>
      {/if}
      {#if activeTab === tab.id}
        <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-accent-primary"></div>
      {/if}
    </button>
  {/each}
</div>
