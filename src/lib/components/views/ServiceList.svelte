<script lang="ts">
  import { onMount } from 'svelte';
  import Badge from '../ui/Badge.svelte';
  import {
    services,
    selectedNamespace,
    currentContext,
    loadServices,
    isLoading,
  } from '../../stores/kubernetes';

  onMount(() => {
    loadServices($selectedNamespace);
    const interval = setInterval(() => loadServices($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    if (!ctx) return; // Don't load if no context is set
    loadServices($selectedNamespace);
  });

  function getServiceTypeVariant(type: string): 'default' | 'success' | 'info' | 'warning' {
    switch (type) {
      case 'LoadBalancer':
        return 'info';
      case 'NodePort':
        return 'warning';
      case 'ClusterIP':
        return 'success';
      default:
        return 'default';
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <div class="p-6 pb-4">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h1 class="text-2xl font-semibold text-text-primary">Services</h1>
        <p class="text-text-muted mt-1">Network services and load balancers</p>
      </div>
      <button
        onclick={() => loadServices($selectedNamespace)}
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
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Type</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Cluster IP</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">External IP</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Ports</th>
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Age</th>
        </tr>
      </thead>
      <tbody>
        {#each $services as service}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{service.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{service.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <Badge variant={getServiceTypeVariant(service.service_type)} size="sm">
                {service.service_type}
              </Badge>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm font-mono">{service.cluster_ip || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm font-mono">{service.external_ip || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <div class="flex flex-wrap gap-1">
                {#each service.ports as port}
                  <span class="text-xs text-text-muted bg-bg-tertiary px-2 py-0.5 rounded font-mono">{port}</span>
                {/each}
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{service.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if $services.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
          </svg>
          <p class="text-text-muted">No services found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
