<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { pvcs, selectedNamespace, currentContext, refreshTrigger, loadPVCs } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');

  async function openDetail(pvc: { name: string; namespace: string }) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'pvc',
        name: pvc.name,
        namespace: pvc.namespace,
        context: $currentContext
      });
    } catch (e) {
      console.error('Failed to open detail:', e);
    }
  }

  const sortedData = $derived(() => {
    const filtered = filterBySearch($pvcs, filterQuery, ['name', 'namespace', 'status', 'volume']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadPVCs($selectedNamespace);
    const interval = setInterval(() => loadPVCs($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
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
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Persistent Volume Claims</h1>
      <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter claims..." />
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Status" field="status" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Volume" field="volume" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Capacity" field="capacity" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Access Modes</th>
          <SortableHeader label="Storage Class" field="storage_class" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as pvc}
          {@const statusDot = pvc.status === 'Bound' ? 'bg-accent-success' : pvc.status === 'Pending' ? 'bg-accent-warning' : 'bg-accent-error'}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer" onclick={() => openDetail(pvc)}>
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {statusDot}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{pvc.name}</span>
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

    {#if sortedData().length === 0}
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
