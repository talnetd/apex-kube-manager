<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import {
    deployments,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadDeployments,
    type DeploymentInfo,
  } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');

  const sortedData = $derived(() => {
    const filtered = filterBySearch($deployments, filterQuery, ['name', 'namespace']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  // Scale modal state
  let showScaleModal = $state<boolean>(false);
  let scaleTarget = $state<DeploymentInfo | null>(null);
  let scaleReplicas = $state<number>(0);
  let isScaling = $state<boolean>(false);

  // Restart state
  let restartingDeployment = $state<string | null>(null);

  onMount(() => {
    loadDeployments($selectedNamespace);
    const interval = setInterval(() => loadDeployments($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadDeployments($selectedNamespace);
  });

  async function openDeploymentDetail(deployment: DeploymentInfo) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'deployment',
        name: deployment.name,
        namespace: deployment.namespace,
        context: $currentContext,
      });
    } catch (e) {
      console.error('Failed to open deployment detail:', e);
    }
  }

  function openScaleModal(deployment: DeploymentInfo, e: Event) {
    e.stopPropagation();
    scaleTarget = deployment;
    // Parse current replicas from ready string (e.g., "2/3" -> 3)
    const parts = deployment.ready.split('/');
    scaleReplicas = parseInt(parts[1]) || 0;
    showScaleModal = true;
  }

  async function handleScale() {
    if (!scaleTarget) return;

    try {
      isScaling = true;
      await invoke('scale_deployment', {
        namespace: scaleTarget.namespace,
        name: scaleTarget.name,
        replicas: scaleReplicas
      });
      showScaleModal = false;
      scaleTarget = null;
      await loadDeployments($selectedNamespace);
    } catch (e) {
      alert(`Failed to scale: ${e}`);
    } finally {
      isScaling = false;
    }
  }

  async function handleRestart(deployment: DeploymentInfo, e: Event) {
    e.stopPropagation();

    if (!confirm(`Are you sure you want to restart deployment "${deployment.name}"?`)) {
      return;
    }

    try {
      restartingDeployment = deployment.name;
      await invoke('restart_deployment', {
        namespace: deployment.namespace,
        name: deployment.name
      });
      await loadDeployments($selectedNamespace);
    } catch (e) {
      alert(`Failed to restart: ${e}`);
    } finally {
      restartingDeployment = null;
    }
  }

  function getReadyStatus(ready: string): 'healthy' | 'degraded' | 'failing' {
    const parts = ready.split('/');
    const current = parseInt(parts[0]) || 0;
    const desired = parseInt(parts[1]) || 0;
    if (desired === 0) return 'failing';
    if (current === desired) return 'healthy';
    if (current > 0) return 'degraded';
    return 'failing';
  }

  function getStatusColor(status: 'healthy' | 'degraded' | 'failing'): string {
    switch (status) {
      case 'healthy': return 'bg-accent-success';
      case 'degraded': return 'bg-accent-warning';
      case 'failing': return 'bg-accent-error';
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Deployments</h1>
      <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter deployments..." />
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
          <SortableHeader label="Ready" field="ready" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Up-to-date" field="up_to_date" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Available" field="available" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-24">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as deployment}
          {@const status = getReadyStatus(deployment.ready)}
          <tr
            class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer"
            onclick={() => openDeploymentDetail(deployment)}
          >
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {getStatusColor(status)}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{deployment.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.ready}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.up_to_date}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.available}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{deployment.age}</span>
            </td>
            <td class="py-3">
              <div class="flex items-center gap-1">
                <button
                  onclick={(e) => openScaleModal(deployment, e)}
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-primary transition-colors"
                  title="Scale"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
                  </svg>
                </button>
                <button
                  onclick={(e) => handleRestart(deployment, e)}
                  disabled={restartingDeployment === deployment.name}
                  class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-warning transition-colors disabled:opacity-50"
                  title="Restart"
                >
                  <svg class="w-4 h-4 {restartingDeployment === deployment.name ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                </button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if sortedData().length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
          </svg>
          <p class="text-text-muted">No deployments found</p>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Scale Modal -->
{#if showScaleModal && scaleTarget}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-bg-secondary rounded-lg p-6 w-96 shadow-xl">
      <h3 class="text-lg font-semibold text-text-primary mb-4">Scale Deployment</h3>
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
