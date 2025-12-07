<script lang="ts">
  import { onMount } from 'svelte';
  import { hpas, selectedNamespace, currentContext, loadHPAs, isLoading } from '../../stores/kubernetes';

  onMount(() => {
    loadHPAs($selectedNamespace);
    const interval = setInterval(() => loadHPAs($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadHPAs($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Horizontal Pod Autoscalers</h1>
        <p class="text-text-muted mt-1">Automatic scaling based on resource utilization</p>
      </div>
      <button onclick={() => loadHPAs($selectedNamespace)} class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors">
        <svg class="w-4 h-4 {$isLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        Refresh
      </button>
    </div>
  </div>

  <div class="flex-1 overflow-auto px-6 pb-6">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Name</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Namespace</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Reference</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Targets</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Min/Max</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Replicas</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $hpas as hpa}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{hpa.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <code class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-accent-primary">{hpa.reference}</code>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.targets}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.min_pods}/{hpa.max_pods}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium">{hpa.replicas}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $hpas.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
          <p class="text-text-muted">No horizontal pod autoscalers found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
