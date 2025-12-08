<script lang="ts">
  import { onMount } from 'svelte';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { hpas, selectedNamespace, currentContext, refreshTrigger, loadHPAs } from '../../stores/kubernetes';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });

  const sortedData = $derived(sortData($hpas, sort.field, sort.direction));

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadHPAs($selectedNamespace);
    const interval = setInterval(() => loadHPAs($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadHPAs($selectedNamespace);
  });
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <h1 class="text-xl font-semibold text-text-primary">Horizontal Pod Autoscalers</h1>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Reference" field="reference" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Targets" field="targets" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Min/Max" field="min_pods" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Replicas" field="replicas" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData as hpa}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{hpa.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <code class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-accent-primary">{hpa.reference}</code>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.targets}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.min_pods}/{hpa.max_pods}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium">{hpa.replicas}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{hpa.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if sortedData.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
          <p class="text-text-muted">No horizontal pod autoscalers found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
