<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import { nodes, nodeMetrics, currentContext, refreshTrigger, startNodeWatch, stopNodeWatch, loadNodeMetrics } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';
  import UsageBar from '../ui/UsageBar.svelte';
  import {
    selectedRowIndex,
    keyboardNavActive,
    totalRows,
    resetNavigation,
  } from '../../stores/keyboard';

  let sort = $state<SortState>({ field: 'name', direction: 'asc' });
  let filterQuery = $state('');
  let tableBody: HTMLTableSectionElement;
  let filterRef: ViewFilter;

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

  const METRICS_INTERVAL = 10000;
  let metricsTimer: ReturnType<typeof setInterval> | null = null;

  function percent(used: number | null | undefined, total: number | null | undefined): number | null {
    if (used == null || !total) return null;
    return (used / total) * 100;
  }

  function formatCpu(millicores: number): string {
    if (millicores >= 1000) return `${(millicores / 1000).toFixed(2)} cores`;
    return `${Math.round(millicores)}m`;
  }

  function formatBytes(bytes: number): string {
    const gib = bytes / 1024 ** 3;
    if (gib >= 1) return `${gib.toFixed(1)} Gi`;
    return `${(bytes / 1024 ** 2).toFixed(0)} Mi`;
  }

  // Usage lives in a separate store (polled), so merge it onto the watched
  // node rows — that also makes the percentages sortable via `sortData`.
  const rows = $derived(() => {
    const metrics = $nodeMetrics;
    return $nodes.map((node) => {
      const m = metrics[node.name];
      return {
        ...node,
        cpu_pct: percent(m?.cpu_usage, m?.cpu_allocatable),
        memory_pct: percent(m?.memory_usage, m?.memory_allocatable),
        disk_pct: percent(m?.disk_usage, m?.disk_capacity),
        cpu_detail: m?.cpu_usage != null ? `${formatCpu(m.cpu_usage)} / ${formatCpu(m.cpu_allocatable)}` : '',
        memory_detail: m?.memory_usage != null ? `${formatBytes(m.memory_usage)} / ${formatBytes(m.memory_allocatable)}` : '',
        disk_detail: m?.disk_usage != null && m?.disk_capacity != null ? `${formatBytes(m.disk_usage)} / ${formatBytes(m.disk_capacity)}` : '',
      };
    });
  });

  const sortedData = $derived(() => {
    const filtered = filterBySearch(rows(), filterQuery, ['name', 'status', 'roles']);
    return sortData(filtered, sort.field, sort.direction);
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  // Get selected node based on current index
  const selectedItem = $derived(() => {
    const items = sortedData();
    const idx = $selectedRowIndex;
    if (idx >= 0 && idx < items.length) {
      return items[idx];
    }
    return null;
  });

  // Update total rows when filtered data changes
  $effect(() => {
    totalRows.set(sortedData().length);
  });

  // Scroll selected row into view
  $effect(() => {
    if ($keyboardNavActive && $selectedRowIndex >= 0 && tableBody) {
      const row = tableBody.children[$selectedRowIndex] as HTMLElement;
      if (row) {
        row.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
      }
    }
  });

  // Keyboard shortcuts for actions
  function handleKeydown(e: KeyboardEvent) {
    const active = document.activeElement;
    const isInput = active?.tagName.toLowerCase() === 'input';

    // Cmd+F to focus filter (works even in input)
    if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
      e.preventDefault();
      filterRef?.focus();
      return;
    }

    // Skip other shortcuts if in input
    if (isInput) return;

    const item = selectedItem();
    if (!item) return;

    // Enter to open detail
    if (e.key === 'Enter') {
      e.preventDefault();
      openDetail(item);
      return;
    }

    // 'y' to copy name
    if (e.key === 'y') {
      e.preventDefault();
      navigator.clipboard.writeText(item.name);
      return;
    }

    // 'r' to refresh
    if (e.key === 'r') {
      e.preventDefault();
      startNodeWatch();
      loadNodeMetrics();
      return;
    }
  }

  onMount(() => {
    startNodeWatch();
    loadNodeMetrics();
    metricsTimer = setInterval(loadNodeMetrics, METRICS_INTERVAL);
    resetNavigation();
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    stopNodeWatch();
    if (metricsTimer) clearInterval(metricsTimer);
    resetNavigation();
    window.removeEventListener('keydown', handleKeydown);
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    startNodeWatch();
    loadNodeMetrics();
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
      <ViewFilter bind:this={filterRef} value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter nodes..." />
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <SortableHeader label="Name" field="name" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Status" field="status" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Roles</th>
          <SortableHeader label="CPU" field="cpu_pct" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Memory" field="memory_pct" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Disk" field="disk_pct" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Version" field="version" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Internal IP" field="internal_ip" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="OS / Runtime" field="os_image" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Taints</th>
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody bind:this={tableBody}>
        {#each sortedData() as node, index}
          {@const isSelected = $keyboardNavActive && $selectedRowIndex === index}
          <tr class="border-b border-border-subtle/50 cursor-pointer transition-colors {isSelected ? 'bg-accent-primary/20 ring-1 ring-accent-primary/50' : 'hover:bg-bg-secondary'}" onclick={() => openDetail(node)}>
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
              <UsageBar percent={node.cpu_pct} detail={node.cpu_detail} unavailableHint="No CPU usage — metrics-server not available" />
            </td>
            <td class="py-3 pr-4">
              <UsageBar percent={node.memory_pct} detail={node.memory_detail} unavailableHint="No memory usage — metrics-server not available" />
            </td>
            <td class="py-3 pr-4">
              <UsageBar percent={node.disk_pct} detail={node.disk_detail} unavailableHint="No disk usage — kubelet stats unreachable (needs nodes/proxy access)" />
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
              {#if node.taints && node.taints.length > 0}
                <div class="flex flex-wrap gap-1">
                  {#each node.taints as taint}
                    <span class="text-xs px-1.5 py-0.5 rounded {taint.effect === 'NoSchedule' ? 'bg-accent-error/10 text-accent-error' : taint.effect === 'PreferNoSchedule' ? 'bg-accent-warning/10 text-accent-warning' : 'bg-accent-primary/10 text-accent-primary'}" title="{taint.key}={taint.value || ''}:{taint.effect}">
                      {taint.key}
                    </span>
                  {/each}
                </div>
              {:else}
                <span class="text-text-muted text-xs">-</span>
              {/if}
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
