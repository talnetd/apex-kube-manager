<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import {
    cronjobs,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadCronJobs,
  } from '../../stores/kubernetes';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });

  async function openDetail(cj: { name: string; namespace: string }) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'cronjob',
        name: cj.name,
        namespace: cj.namespace,
        context: $currentContext
      });
    } catch (e) {
      console.error('Failed to open detail:', e);
    }
  }

  const sortedData = $derived(sortData($cronjobs, sort.field, sort.direction));

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadCronJobs($selectedNamespace);
    const interval = setInterval(() => loadCronJobs($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadCronJobs($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <h1 class="text-xl font-semibold text-text-primary">CronJobs</h1>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Schedule" field="schedule" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Suspend" field="suspend" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Active" field="active" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Last Schedule" field="last_schedule" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-24">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each sortedData as cj}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer" onclick={() => openDetail(cj)}>
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {cj.suspend ? 'bg-accent-warning' : 'bg-accent-success'}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{cj.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <code class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-accent-primary">{cj.schedule}</code>
            </td>
            <td class="py-3 pr-4">
              {#if cj.suspend}
                <span class="px-2 py-0.5 rounded text-xs font-medium text-accent-warning bg-accent-warning/10">Suspended</span>
              {:else}
                <span class="px-2 py-0.5 rounded text-xs font-medium text-accent-success bg-accent-success/10">Active</span>
              {/if}
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.active}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.last_schedule || 'Never'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{cj.age}</span>
            </td>
            <td class="py-3">
              <div class="flex items-center gap-1">
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-success transition-colors"
                  title="Trigger Now"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </button>
                <button
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-warning transition-colors"
                  title="{cj.suspend ? 'Resume' : 'Suspend'}"
                >
                  {#if cj.suspend}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    </svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                  {/if}
                </button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if sortedData.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-text-muted">No cronjobs found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
