<script lang="ts">
  import { onMount } from 'svelte';
  import { networkPolicies, selectedNamespace, currentContext, loadNetworkPolicies, isLoading } from '../../stores/kubernetes';

  onMount(() => {
    loadNetworkPolicies($selectedNamespace);
    const interval = setInterval(() => loadNetworkPolicies($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadNetworkPolicies($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Network Policies</h1>
        <p class="text-text-muted mt-1">Pod-level network traffic rules</p>
      </div>
      <button onclick={() => loadNetworkPolicies($selectedNamespace)} class="flex items-center gap-2 px-4 py-2 bg-bg-tertiary border border-border-subtle rounded-lg text-text-primary hover:border-accent-primary transition-colors">
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
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Pod Selector</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Policy Types</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $networkPolicies as np}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{np.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{np.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <code class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-text-secondary">{np.pod_selector || '<all>'}</code>
            </td>
            <td class="py-3 pr-4">
              <div class="flex gap-1">
                {#each np.policy_types as ptype}
                  <span class="text-xs px-2 py-0.5 rounded {ptype === 'Ingress' ? 'bg-accent-success/10 text-accent-success' : 'bg-accent-warning/10 text-accent-warning'}">{ptype}</span>
                {/each}
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{np.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $networkPolicies.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
          </svg>
          <p class="text-text-muted">No network policies found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
