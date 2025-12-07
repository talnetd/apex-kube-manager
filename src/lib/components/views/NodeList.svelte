<script lang="ts">
  import { onMount } from 'svelte';
  import { nodes, currentContext, loadNodes, isLoading } from '../../stores/kubernetes';

  onMount(() => {
    loadNodes();
    const interval = setInterval(() => loadNodes(), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadNodes();
  });

  function getStatusColor(status: string): string {
    if (status.includes('Ready') && !status.includes('NotReady')) {
      return status.includes('SchedulingDisabled')
        ? 'text-accent-warning bg-accent-warning/10'
        : 'text-accent-success bg-accent-success/10';
    }
    return 'text-accent-error bg-accent-error/10';
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Nodes</h1>
        <p class="text-text-muted mt-1">Worker machines in the cluster</p>
      </div>
      <button onclick={() => loadNodes()} class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors">
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
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Status</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Roles</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Version</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Internal IP</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">OS / Runtime</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $nodes as node}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <div class="flex items-center gap-2">
                <div class="w-2 h-2 rounded-full {node.status.includes('Ready') && !node.status.includes('NotReady') ? 'bg-accent-success' : 'bg-accent-error'}"></div>
                <span class="text-text-primary font-medium">{node.name}</span>
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-xs px-2 py-0.5 rounded {getStatusColor(node.status)}">{node.status}</span>
            </td>
            <td class="py-3 pr-4">
              <div class="flex flex-wrap gap-1">
                {#each node.roles as role}
                  <span class="text-xs bg-accent-primary/10 text-accent-primary px-2 py-0.5 rounded">{role}</span>
                {/each}
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{node.version}</span>
            </td>
            <td class="py-3 pr-4">
              <code class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-text-secondary">{node.internal_ip || '-'}</code>
            </td>
            <td class="py-3 pr-4">
              <div class="text-text-secondary text-xs">
                <div>{node.os_image}</div>
                <div class="text-text-muted">{node.container_runtime}</div>
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{node.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $nodes.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
          </svg>
          <p class="text-text-muted">No nodes found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
