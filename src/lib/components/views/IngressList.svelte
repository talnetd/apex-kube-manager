<script lang="ts">
  import { onMount } from 'svelte';
  import { ingresses, selectedNamespace, currentContext, loadIngresses, isLoading } from '../../stores/kubernetes';

  onMount(() => {
    loadIngresses($selectedNamespace);
    const interval = setInterval(() => loadIngresses($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadIngresses($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Ingresses</h1>
        <p class="text-text-muted mt-1">External access to services via HTTP/HTTPS</p>
      </div>
      <button onclick={() => loadIngresses($selectedNamespace)} class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors">
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
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Class</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Hosts</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Address</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Ports</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $ingresses as ing}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{ing.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{ing.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{ing.class || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <div class="flex flex-wrap gap-1">
                {#each ing.hosts as host}
                  <span class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-accent-primary">{host}</span>
                {/each}
                {#if ing.hosts.length === 0}
                  <span class="text-text-muted text-sm">*</span>
                {/if}
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{ing.address || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{ing.ports}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{ing.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $ingresses.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
          </svg>
          <p class="text-text-muted">No ingresses found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
