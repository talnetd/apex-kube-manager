<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import {
    replicasets,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadReplicaSets,
  } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');

  async function openDetail(rs: { name: string; namespace: string }) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'replicaset',
        name: rs.name,
        namespace: rs.namespace,
        context: $currentContext
      });
    } catch (e) {
      console.error('Failed to open detail:', e);
    }
  }

  const sortedData = $derived(() => {
    const filtered = filterBySearch($replicasets, filterQuery, ['name', 'namespace', 'owner']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadReplicaSets($selectedNamespace);
    const interval = setInterval(() => loadReplicaSets($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
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
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">ReplicaSets</h1>
      <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter replicasets..." />
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
          <SortableHeader label="Desired" field="desired" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Current" field="current" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Ready" field="ready" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Owner" field="owner" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as rs}
          {@const status = getReadyStatus(rs.ready, rs.desired)}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer" onclick={() => openDetail(rs)}>
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {status === 'healthy' ? 'bg-accent-success' : status === 'degraded' ? 'bg-accent-warning' : 'bg-accent-error'}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{rs.name}</span>
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

    {#if sortedData().length === 0}
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
