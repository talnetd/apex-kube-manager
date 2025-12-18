<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import {
    statefulsets,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadStatefulSets,
    type StatefulSetInfo,
  } from '../../stores/kubernetes';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });

  const sortedData = $derived(sortData($statefulsets, sort.field, sort.direction));

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  // Scale modal state
  let showScaleModal = $state<boolean>(false);
  let scaleTarget = $state<StatefulSetInfo | null>(null);
  let scaleReplicas = $state<number>(0);
  let isScaling = $state<boolean>(false);

  // Restart state
  let restartingStatefulSet = $state<string | null>(null);

  onMount(() => {
    loadStatefulSets($selectedNamespace);
    const interval = setInterval(() => loadStatefulSets($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadStatefulSets($selectedNamespace);
  });

  async function openStatefulSetDetail(sts: StatefulSetInfo) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'statefulset',
        name: sts.name,
        namespace: sts.namespace,
        context: $currentContext,
      });
    } catch (e) {
      console.error('Failed to open statefulset detail:', e);
    }
  }

  function openScaleModal(sts: StatefulSetInfo, e: Event) {
    e.stopPropagation();
    scaleTarget = sts;
    scaleReplicas = sts.replicas;
    showScaleModal = true;
  }

  async function handleScale() {
    if (!scaleTarget) return;

    try {
      isScaling = true;
      await invoke('scale_statefulset', {
        namespace: scaleTarget.namespace,
        name: scaleTarget.name,
        replicas: scaleReplicas
      });
      showScaleModal = false;
      scaleTarget = null;
      await loadStatefulSets($selectedNamespace);
    } catch (e) {
      alert(`Failed to scale: ${e}`);
    } finally {
      isScaling = false;
    }
  }

  async function handleRestart(sts: StatefulSetInfo, e: Event) {
    e.stopPropagation();

    if (!confirm(`Are you sure you want to restart statefulset "${sts.name}"?`)) {
      return;
    }

    try {
      restartingStatefulSet = sts.name;
      await invoke('restart_statefulset', {
        namespace: sts.namespace,
        name: sts.name
      });
      await loadStatefulSets($selectedNamespace);
    } catch (e) {
      alert(`Failed to restart: ${e}`);
    } finally {
      restartingStatefulSet = null;
    }
  }

  function getReadyStatus(ready: string): 'healthy' | 'degraded' | 'down' {
    const [current, total] = ready.split('/').map(Number);
    if (current === total && total > 0) return 'healthy';
    if (current > 0) return 'degraded';
    return 'down';
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <h1 class="text-xl font-semibold text-text-primary">StatefulSets</h1>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Ready" field="ready" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Replicas" field="replicas" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Service" field="service_name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-24">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each sortedData as sts}
          {@const status = getReadyStatus(sts.ready)}
          <tr
            class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer"
            onclick={() => openStatefulSetDetail(sts)}
          >
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {status === 'healthy' ? 'bg-accent-success' : status === 'degraded' ? 'bg-accent-warning' : 'bg-accent-error'}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{sts.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sts.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-sm {status === 'healthy' ? 'text-accent-success' : status === 'degraded' ? 'text-accent-warning' : 'text-accent-error'}">{sts.ready}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sts.replicas}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sts.service_name || '-'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sts.age}</span>
            </td>
            <td class="py-3">
              <div class="flex items-center gap-1">
                <button
                  onclick={(e) => openScaleModal(sts, e)}
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-primary transition-colors"
                  title="Scale"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                  </svg>
                </button>
                <button
                  onclick={(e) => handleRestart(sts, e)}
                  disabled={restartingStatefulSet === sts.name}
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-warning transition-colors disabled:opacity-50"
                  title="Restart"
                >
                  <svg class="w-4 h-4 {restartingStatefulSet === sts.name ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
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
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <p class="text-text-muted">No statefulsets found</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Scale Modal -->
{#if showScaleModal && scaleTarget}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-bg-secondary rounded-lg p-6 w-96 shadow-xl">
      <h3 class="text-lg font-semibold text-text-primary mb-4">Scale StatefulSet</h3>
      <p class="text-sm text-text-secondary mb-4">
        Set the number of replicas for <span class="text-accent-primary">{scaleTarget.name}</span>
      </p>
      <div class="flex items-center justify-center gap-3 mb-6">
        <button
          onclick={() => scaleReplicas = Math.max(0, scaleReplicas - 1)}
          class="w-12 h-12 rounded-lg bg-bg-tertiary hover:bg-accent-error/20 hover:text-accent-error border border-border-subtle text-text-primary transition-colors flex items-center justify-center"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
          </svg>
        </button>
        <input
          type="number"
          bind:value={scaleReplicas}
          min="0"
          class="w-24 text-center text-3xl font-bold bg-bg-primary border border-border-subtle rounded-lg py-3 text-text-primary focus:outline-none focus:border-accent-primary [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
        />
        <button
          onclick={() => scaleReplicas++}
          class="w-12 h-12 rounded-lg bg-bg-tertiary hover:bg-accent-success/20 hover:text-accent-success border border-border-subtle text-text-primary transition-colors flex items-center justify-center"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
        </button>
      </div>
      <div class="flex justify-end gap-3">
        <button
          onclick={() => { showScaleModal = false; scaleTarget = null; }}
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleScale}
          disabled={isScaling}
          class="px-4 py-2 text-sm bg-accent-primary text-white rounded hover:bg-accent-primary/90 transition-colors disabled:opacity-50"
        >
          {isScaling ? 'Scaling...' : 'Scale'}
        </button>
      </div>
    </div>
  </div>
{/if}
