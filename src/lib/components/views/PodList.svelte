<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import StatusBadge from '../ui/StatusBadge.svelte';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import {
    pods,
    podsByStatus,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadPods,
    deletePod,
    openTerminalWindow,
    isLoading,
  } from '../../stores/kubernetes';
  import type { PodInfo } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';

  let activeFilter = $state('all');
  let showDeleteConfirm = $state(false);
  let podToDelete = $state<PodInfo | null>(null);
  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');

  const filters = $derived([
    { id: 'all', label: 'All', count: $podsByStatus.all.length },
    { id: 'running', label: 'Running', count: $podsByStatus.running.length },
    { id: 'pending', label: 'Pending', count: $podsByStatus.pending.length },
    { id: 'failed', label: 'Failed', count: $podsByStatus.failed.length },
  ]);

  const filteredPods = $derived(() => {
    let data: PodInfo[];
    switch (activeFilter) {
      case 'running':
        data = $podsByStatus.running;
        break;
      case 'pending':
        data = $podsByStatus.pending;
        break;
      case 'failed':
        data = $podsByStatus.failed;
        break;
      default:
        data = $podsByStatus.all;
    }
    // Apply text search filter
    const searched = filterBySearch(data, filterQuery, ['name', 'namespace', 'status', 'node']);
    return sortData(searched, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadPods($selectedNamespace);
    const interval = setInterval(() => loadPods($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  // React to namespace, context, and manual refresh trigger
  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadPods($selectedNamespace);
  });

  async function openPodDetail(pod: PodInfo) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'pod',
        name: pod.name,
        namespace: pod.namespace,
        context: $currentContext,
      });
    } catch (e) {
      console.error('Failed to open pod detail:', e);
    }
  }

  function confirmDelete(pod: PodInfo) {
    podToDelete = pod;
    showDeleteConfirm = true;
  }

  async function executeDelete() {
    if (podToDelete) {
      await deletePod(podToDelete.namespace, podToDelete.name);
      showDeleteConfirm = false;
      podToDelete = null;
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Pods</h1>
      <div class="flex items-center gap-3">
        <!-- Filter Pills -->
        <div class="flex items-center gap-1">
          {#each filters as filter}
            <button
              onclick={() => activeFilter = filter.id}
              class="px-2.5 py-1 text-sm rounded-md transition-colors
                {activeFilter === filter.id
                  ? 'bg-accent-primary/20 text-accent-primary border border-accent-primary/30'
                  : 'bg-bg-tertiary text-text-secondary hover:text-text-primary border border-transparent'}"
            >
              {filter.label}
              <span class="ml-1 text-xs opacity-70">({filter.count})</span>
            </button>
          {/each}
        </div>
        <!-- Text Filter -->
        <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter pods..." />
      </div>
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
          <SortableHeader label="Ready" field="ready" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Restarts" field="restarts" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-24">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each filteredPods() as pod}
          {@const statusColor = pod.status === 'Running' ? 'bg-accent-success' : pod.status === 'Pending' ? 'bg-accent-warning' : 'bg-accent-error'}
          <tr
            onclick={() => openPodDetail(pod)}
            class="border-b border-border-subtle/50 cursor-pointer transition-colors hover:bg-bg-secondary"
          >
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {statusColor}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{pod.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pod.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <StatusBadge status={pod.status} />
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pod.ready}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-sm {pod.restarts > 0 ? 'text-accent-warning' : 'text-text-secondary'}">
                {pod.restarts}
              </span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{pod.age}</span>
            </td>
            <td class="py-3">
              <div class="flex items-center gap-1">
                <button
                  onclick={(e) => { e.stopPropagation(); openTerminalWindow(pod.namespace, pod.name); }}
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-primary transition-colors"
                  title="Open Terminal"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                </button>
                <button
                  onclick={(e) => { e.stopPropagation(); confirmDelete(pod); }}
                  class="p-1.5 rounded hover:bg-accent-error/20 text-text-muted hover:text-accent-error transition-colors"
                  title="Delete Pod"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if filteredPods().length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p class="text-text-muted">No pods found</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm && podToDelete}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-bg-card border border-border-subtle rounded-lg p-6 max-w-md w-full mx-4">
      <h3 class="text-lg font-semibold text-text-primary mb-2">Delete Pod</h3>
      <p class="text-text-secondary mb-4">
        Are you sure you want to delete <span class="text-text-primary font-medium">{podToDelete.name}</span>?
        This action cannot be undone.
      </p>
      <div class="flex items-center justify-end gap-3">
        <button
          onclick={() => { showDeleteConfirm = false; podToDelete = null; }}
          class="px-4 py-2 text-text-secondary hover:text-text-primary transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={executeDelete}
          class="px-4 py-2 bg-accent-error text-white rounded-lg hover:bg-accent-error/80 transition-colors"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}
