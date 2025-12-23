<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import SortableHeader from '../ui/SortableHeader.svelte';
  import { sortData, toggleSort, type SortState } from '../../utils/sort';
  import {
    clusterEvents,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    startEventWatch,
    stopEventWatch,
    type ClusterEventInfo,
  } from '../../stores/kubernetes';
  import { filterBySearch } from '../../stores/search';
  import ViewFilter from '../ui/ViewFilter.svelte';
  import CustomSelect from '../ui/CustomSelect.svelte';

  let sort = $state<SortState>({ field: 'age', direction: 'asc' });
  let filterQuery = $state('');
  let typeFilter = $state<'all' | 'Normal' | 'Warning'>('all');
  let reasonFilter = $state<string>('all');

  // Get unique reasons from events
  const uniqueReasons = $derived(() => {
    const reasons = new Set($clusterEvents.map(e => e.reason).filter(Boolean));
    return Array.from(reasons).sort();
  });

  const filteredData = $derived(() => {
    let filtered = $clusterEvents;
    if (typeFilter !== 'all') {
      filtered = filtered.filter(e => e.type === typeFilter);
    }
    if (reasonFilter !== 'all') {
      filtered = filtered.filter(e => e.reason === reasonFilter);
    }
    return filtered;
  });

  const sortedData = $derived(() => {
    const filtered = filterBySearch(filteredData(), filterQuery, ['reason', 'message', 'involved_object', 'involved_kind', 'namespace']);
    return sortData(filtered, sort.field, sort.direction);
  });

  const eventCounts = $derived(() => {
    const all = $clusterEvents.length;
    const normal = $clusterEvents.filter(e => e.type === 'Normal').length;
    const warning = $clusterEvents.filter(e => e.type === 'Warning').length;
    return { all, normal, warning };
  });

  function handleSort(field: string) {
    sort = toggleSort(sort, field);
  }

  onMount(() => {
    startEventWatch($selectedNamespace);
  });

  onDestroy(() => {
    stopEventWatch();
  });

  $effect(() => {
    const ctx = $currentContext;
    const trigger = $refreshTrigger;
    if (!ctx) return;
    startEventWatch($selectedNamespace);
  });

  function getTypeColor(type: string): string {
    return type === 'Warning'
      ? 'text-accent-warning bg-accent-warning/10'
      : 'text-accent-success bg-accent-success/10';
  }

  function getTypeDot(type: string): string {
    return type === 'Warning' ? 'bg-accent-warning' : 'bg-accent-success';
  }

  let copiedName = $state<string | null>(null);

  async function copyMessage(event: ClusterEventInfo) {
    await navigator.clipboard.writeText(event.message);
    copiedName = event.name;
    setTimeout(() => copiedName = null, 1500);
  }

  // Map K8s kind to our resource type names
  const kindToResourceType: Record<string, string> = {
    'Pod': 'pod',
    'Deployment': 'deployment',
    'StatefulSet': 'statefulset',
    'DaemonSet': 'daemonset',
    'ReplicaSet': 'replicaset',
    'Job': 'job',
    'CronJob': 'cronjob',
    'Service': 'service',
    'Ingress': 'ingress',
    'ConfigMap': 'configmap',
    'Secret': 'secret',
    'PersistentVolume': 'pv',
    'PersistentVolumeClaim': 'pvc',
    'Node': 'node',
    'Namespace': 'namespace',
    'ServiceAccount': 'serviceaccount',
    'HorizontalPodAutoscaler': 'hpa',
    'NetworkPolicy': 'networkpolicy',
  };

  async function openResourceDetail(event: ClusterEventInfo) {
    const resourceType = kindToResourceType[event.involved_kind];
    if (!resourceType || !event.involved_object) return;

    try {
      await invoke('open_resource_detail', {
        resourceType,
        name: event.involved_object,
        namespace: event.namespace || '',
        context: $currentContext,
      });
    } catch (e) {
      console.error('Failed to open resource detail:', e);
    }
  }

  function isClickable(kind: string): boolean {
    return kind in kindToResourceType;
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Toolbar -->
  <div class="px-6 py-4 border-b border-border-subtle">
    <div class="flex items-center justify-between">
      <h1 class="text-xl font-semibold text-text-primary">Events</h1>
      <div class="flex items-center gap-3">
        <!-- Type Filter Pills -->
        <div class="flex items-center gap-1">
          <button
            onclick={() => typeFilter = 'all'}
            class="px-2.5 py-1 text-sm rounded-md transition-colors
              {typeFilter === 'all'
                ? 'bg-accent-primary/20 text-accent-primary border border-accent-primary/30'
                : 'bg-bg-tertiary text-text-secondary hover:text-text-primary border border-transparent'}"
          >
            All
            <span class="ml-1 text-xs opacity-70">({eventCounts().all})</span>
          </button>
          <button
            onclick={() => typeFilter = 'Normal'}
            class="px-2.5 py-1 text-sm rounded-md transition-colors
              {typeFilter === 'Normal'
                ? 'bg-accent-success/20 text-accent-success border border-accent-success/30'
                : 'bg-bg-tertiary text-text-secondary hover:text-text-primary border border-transparent'}"
          >
            Normal
            <span class="ml-1 text-xs opacity-70">({eventCounts().normal})</span>
          </button>
          <button
            onclick={() => typeFilter = 'Warning'}
            class="px-2.5 py-1 text-sm rounded-md transition-colors
              {typeFilter === 'Warning'
                ? 'bg-accent-warning/20 text-accent-warning border border-accent-warning/30'
                : 'bg-bg-tertiary text-text-secondary hover:text-text-primary border border-transparent'}"
          >
            Warning
            <span class="ml-1 text-xs opacity-70">({eventCounts().warning})</span>
          </button>
        </div>
        <!-- Reason Filter -->
        <div class="w-40">
          <CustomSelect
            value={reasonFilter}
            options={[{ value: 'all', label: 'All Reasons' }, ...uniqueReasons().map(r => ({ value: r, label: r.length > 18 ? r.slice(0, 18) + '…' : r }))]}
            placeholder="Reason..."
            onchange={(v) => reasonFilter = v}
          />
        </div>
        <!-- Text Filter -->
        <ViewFilter value={filterQuery} onchange={(v) => filterQuery = v} placeholder="Filter events..." />
      </div>
    </div>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto p-6 pt-4">
    <table class="w-full">
      <thead>
        <tr class="text-left border-b border-border-subtle">
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium w-4"></th>
          <SortableHeader label="Type" field="type" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Reason" field="reason" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Object" field="involved_object" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Namespace" field="namespace" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <th class="pb-3 text-xs text-text-muted uppercase tracking-wide font-medium">Message</th>
          <SortableHeader label="Count" field="count" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
          <SortableHeader label="Age" field="age" sortField={sort.field} sortDirection={sort.direction} onSort={handleSort} />
        </tr>
      </thead>
      <tbody>
        {#each sortedData() as event}
          <tr class="border-b border-border-subtle/50 hover:bg-bg-secondary transition-colors">
            <td class="py-3 pr-2">
              <div class="w-2 h-2 rounded-full {getTypeDot(event.type)}"></div>
            </td>
            <td class="py-3 pr-4">
              <span class="px-2 py-0.5 rounded text-xs font-medium {getTypeColor(event.type)}">{event.type}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-primary font-medium">{event.reason}</span>
            </td>
            <td class="py-3 pr-4">
              <div class="flex items-center gap-1.5">
                <span class="text-xs bg-bg-tertiary text-text-muted px-1.5 py-0.5 rounded">{event.involved_kind}</span>
                {#if isClickable(event.involved_kind)}
                  <button
                    onclick={() => openResourceDetail(event)}
                    class="text-accent-primary text-sm hover:underline hover:text-accent-primary/80 transition-colors text-left"
                  >
                    {event.involved_object}
                  </button>
                {:else}
                  <span class="text-text-secondary text-sm">{event.involved_object}</span>
                {/if}
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{event.namespace || '-'}</span>
            </td>
            <td class="py-3 pr-4 max-w-md">
              <div class="flex items-start gap-2 group">
                <span class="text-text-secondary text-sm line-clamp-2 flex-1" title={event.message}>{event.message}</span>
                <button
                  onclick={() => copyMessage(event)}
                  class="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-bg-tertiary transition-all shrink-0"
                  title="Copy message"
                >
                  {#if copiedName === event.name}
                    <svg class="w-4 h-4 text-accent-success" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                  {:else}
                    <svg class="w-4 h-4 text-text-muted hover:text-text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                  {/if}
                </button>
              </div>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{event.count}</span>
            </td>
            <td class="py-3 pr-4">
              <span class="text-text-secondary text-sm">{event.age}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#if sortedData().length === 0}
      <div class="flex items-center justify-center h-48">
        <div class="text-center">
          <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-text-muted">No events found</p>
        </div>
      </div>
    {/if}
  </div>
</div>
