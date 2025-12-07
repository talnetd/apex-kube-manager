<script lang="ts">
  import { onMount } from 'svelte';
  import {
    replicasets,
    selectedNamespace,
    currentContext,
    loadReplicaSets,
    isLoading,
  } from '../../stores/kubernetes';

  onMount(() => {
    loadReplicaSets($selectedNamespace);
    const interval = setInterval(() => loadReplicaSets($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadReplicaSets($selectedNamespace);
  });

  function getReadyStatus(ready: number, desired: number): 'healthy' | 'degraded' | 'down' {
    if (desired === 0) return 'healthy';
    if (ready === desired) return 'healthy';
    if (ready > 0) return 'degraded';
    return 'down';
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">ReplicaSets</h1>
        <p class="text-text-muted mt-1">Maintain stable set of replica pods</p>
      </div>
      <button
        onclick={() => loadReplicaSets($selectedNamespace)}
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
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Desired</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Current</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Ready</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Owner</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $replicasets as rs}
          {@const status = getReadyStatus(rs.ready, rs.desired)}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {status === 'healthy' ? 'bg-accent-success' : status === 'degraded' ? 'bg-accent-warning' : 'bg-accent-error'}"></div>
                <span class="text-text-primary font-medium">{rs.name}</span>
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{rs.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{rs.desired}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{rs.current}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-sm {status === 'healthy' ? 'text-accent-success' : status === 'degraded' ? 'text-accent-warning' : 'text-accent-error'}">{rs.ready}/{rs.desired}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-muted text-sm">{rs.owner || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{rs.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $replicasets.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p class="text-text-muted">No replicasets found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
