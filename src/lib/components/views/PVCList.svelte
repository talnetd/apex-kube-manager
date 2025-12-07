<script lang="ts">
  import { onMount } from 'svelte';
  import { pvcs, selectedNamespace, currentContext, loadPVCs, isLoading } from '../../stores/kubernetes';

  onMount(() => {
    loadPVCs($selectedNamespace);
    const interval = setInterval(() => loadPVCs($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadPVCs($selectedNamespace);
  });

  function getStatusColor(status: string): string {
    switch (status) {
      case 'Bound': return 'text-accent-success bg-accent-success/10';
      case 'Pending': return 'text-accent-warning bg-accent-warning/10';
      case 'Lost': return 'text-accent-error bg-accent-error/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Persistent Volume Claims</h1>
        <p class="text-text-muted mt-1">Storage requests by pods</p>
      </div>
      <button onclick={() => loadPVCs($selectedNamespace)} class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors">
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
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Status</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Volume</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Capacity</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Access Modes</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Storage Class</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $pvcs as pvc}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{pvc.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pvc.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-xs px-2 py-0.5 rounded {getStatusColor(pvc.status)}">{pvc.status}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pvc.volume || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium">{pvc.capacity || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pvc.access_modes.join(', ') || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pvc.storage_class || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pvc.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $pvcs.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
          </svg>
          <p class="text-text-muted">No persistent volume claims found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
