<script lang="ts">
  import { onMount } from 'svelte';
  import Badge from '../ui/Badge.svelte';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import {
    services,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadServices,
  } from '../../stores/kubernetes';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });

  const sortedData = $derived(sortData($services, sort.field, sort.direction));

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadServices($selectedNamespace);
    const interval = setInterval(() => loadServices($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
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
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <h1 class="text-xl font-semibold text-text-primary">Services</h1>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Type" field="service_type" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Cluster IP" field="cluster_ip" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="External IP" field="external_ip" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Ports</th>
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData as service}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer">
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full bg-accent-success"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{service.name}</span>
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

    {#if sortedData.length === 0}
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
