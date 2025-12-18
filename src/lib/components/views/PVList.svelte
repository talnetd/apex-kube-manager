<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { pvs, currentContext, refreshTrigger, loadPVs } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');

  async function openDetail(pv: { name: string }) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'pv',
        name: pv.name,
        namespace: '',
        context: $currentContext
      });
    } catch (e) {
      console.error('Failed to open detail:', e);
    }
  }

  const sortedData = $derived(() => {
    const filtered = filterBySearch($pvs, filterQuery, ['name', 'status', 'storage_class']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadPVs();
    const interval = setInterval(() => loadPVs(), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadPVs();
  });

  function getStatusColor(status: string): string {
    switch (status) {
      case 'Available': return 'text-accent-success bg-accent-success/10';
      case 'Bound': return 'text-accent-primary bg-accent-primary/10';
      case 'Released': return 'text-accent-warning bg-accent-warning/10';
      case 'Failed': return 'text-accent-error bg-accent-error/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Persistent Volumes</h1>
      <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter volumes..." />
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Capacity" field="capacity" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Access Modes</th>
          <SortableHeader label="Reclaim" field="reclaim_policy" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Status" field="status" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Claim" field="claim" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Storage Class" field="storage_class" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as pv}
          {@const statusDot = pv.status === 'Bound' ? 'bg-accent-primary' : pv.status === 'Available' ? 'bg-accent-success' : pv.status === 'Released' ? 'bg-accent-warning' : 'bg-accent-error'}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer" onclick={() => openDetail(pv)}>
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {statusDot}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{pv.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium">{pv.capacity}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pv.access_modes.join(', ')}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pv.reclaim_policy}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-xs px-2 py-0.5 rounded {getStatusColor(pv.status)}">{pv.status}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pv.claim || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pv.storage_class || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pv.age}</span>
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
          <p class="text-text-muted">No persistent volumes found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
