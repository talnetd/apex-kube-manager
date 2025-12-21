<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { nodes, currentContext, refreshTrigger, startNodeWatch, stopNodeWatch } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');

  async function openDetail(node: { name: string }) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'node',
        name: node.name,
        namespace: '',
        context: $currentContext
      });
    } catch (e) {
      console.error('Failed to open detail:', e);
    }
  }

  const sortedData = $derived(() => {
    const filtered = filterBySearch($nodes, filterQuery, ['name', 'status', 'roles']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    startNodeWatch();
  });

  onDestroy(() => {
    stopNodeWatch();
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    startNodeWatch();
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
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Nodes</h1>
      <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter nodes..." />
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Status" field="status" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Roles</th>
          <SortableHeader label="Version" field="version" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Internal IP" field="internal_ip" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="OS / Runtime" field="os_image" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as node}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer" onclick={() => openDetail(node)}>
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {node.status.includes('Ready') && !node.status.includes('NotReady') ? 'bg-accent-success' : 'bg-accent-error'}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{node.name}</span>
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

    {#if sortedData().length === 0}
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
