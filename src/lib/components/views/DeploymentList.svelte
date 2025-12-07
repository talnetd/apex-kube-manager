<script lang="ts">
  import { onMount } from 'svelte';
  import StatusBadge from '../ui/StatusBadge.svelte';
  import {
    deployments,
    selectedNamespace,
    currentContext,
    loadDeployments,
    isLoading,
  } from '../../stores/kubernetes';

  onMount(() => {
    loadDeployments($selectedNamespace);
    const interval = setInterval(() => loadDeployments($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadDeployments($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Deployments</h1>
        <p class="text-text-muted mt-1">Manage your application deployments</p>
      </div>
      <button
        onclick={() => loadDeployments($selectedNamespace)}
        class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors"
      >
        <svg class="w-4 h-4 {$isLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        Refresh
      </button>
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto px-6 pb-6">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Name</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Namespace</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Ready</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Up-to-date</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Available</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-20"></th>
        </tr>
      </thead>
      <tbody>
        {#each $deployments as deployment}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{deployment.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.ready}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.up_to_date}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.available}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.age}</span>
            </td>
            <td class="py-3">
              <div class="flex items-center gap-1">
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
                  title="Scale"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                  </svg>
                </button>
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
                  title="Restart"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                </button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $deployments.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
          </svg>
          <p class="text-text-muted">No deployments found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
