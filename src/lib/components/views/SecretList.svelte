<script lang="ts">
  import { onMount } from 'svelte';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { secrets, selectedNamespace, currentContext, refreshTrigger, loadSecrets } from '../../stores/kubernetes';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });

  const sortedData = $derived(sortData($secrets, sort.field, sort.direction));

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    loadSecrets($selectedNamespace);
    const interval = setInterval(() => loadSecrets($selectedNamespace), 10000);
    return () => clearInterval(interval);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    loadSecrets($selectedNamespace);
  });

  function getTypeColor(type: string): string {
    if (type.includes('tls')) return 'text-accent-primary bg-accent-primary/10';
    if (type.includes('dockerconfig')) return 'text-accent-warning bg-accent-warning/10';
    if (type.includes('service-account')) return 'text-accent-success bg-accent-success/10';
    return 'text-text-muted bg-bg-tertiary';
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <h1 class="text-xl font-semibold text-text-primary">Secrets</h1>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Type" field="secret_type" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Data" field="data_count" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData as sec}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors cursor-pointer">
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full bg-accent-warning"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-accent-primary font-medium hover:underline">{sec.name}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sec.namespace}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-xs px-2 py-0.5 rounded {getTypeColor(sec.secret_type)}">{sec.secret_type}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sec.data_count} {sec.data_count === 1 ? 'key' : 'keys'}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{sec.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if sortedData.length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
          <p class="text-text-muted">No secrets found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
