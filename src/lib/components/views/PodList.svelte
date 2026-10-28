<script lang="ts">
  // Pods: the reference list view (redesign PR1b). PageHeader + DataTable +
  // StatusPill, health from the Rust verdict only. DataTable owns keyboard
  // nav, selection anchoring, Enter/y/r and `d` (opens detail: lists never
  // delete, CLAUDE.md Architecture Note 7). Pod delete lives in PodDetail.
  import { onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import PageHeader, { type Segment } from '../ui/PageHeader.svelte';
  import DataTable from '../ui/DataTable.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import StatusIcon from '../ui/StatusIcon.svelte';
  import IconButton from '../ui/IconButton.svelte';
  import Button from '../ui/Button.svelte';
  import { HEALTH_SORT_ID, type Column, type TableState } from '../ui/data-table';
  import { toDisplay, type HealthResult } from '../../utils/health';
  import {
    podsByStatus,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    startPodWatch,
    stopPodWatch,
    openTerminalWindow,
    resourceState,
    type PodInfo,
  } from '../../stores/kubernetes';

  type SegmentId = 'all' | 'failed' | 'warning' | 'progressing' | 'healthy' | 'completed';

  let activeSegment = $state<SegmentId>('all');
  let filter = $state('');

  const loadState = resourceState('pods');

  const segments = $derived<Segment[]>([
    { id: 'all', label: 'All', count: $podsByStatus.all.length },
    { id: 'failed', label: 'Failing', count: $podsByStatus.failed.length, health: 'failed' },
    { id: 'warning', label: 'Warning', count: $podsByStatus.warning.length, health: 'warning' },
    { id: 'progressing', label: 'Pending', count: $podsByStatus.progressing.length, health: 'progressing' },
    { id: 'healthy', label: 'Running', count: $podsByStatus.healthy.length, health: 'healthy' },
    { id: 'completed', label: 'Completed', count: $podsByStatus.completed.length, health: 'neutral' },
  ]);

  // A segment that empties out (last failing pod recovered) falls back to All.
  $effect(() => {
    if (activeSegment !== 'all' && $podsByStatus[activeSegment].length === 0) activeSegment = 'all';
  });

  function matches(p: PodInfo, q: string): boolean {
    return [p.name, p.namespace, p.node ?? '', p.status, p.verdict.reason].some((v) => v.toLowerCase().includes(q));
  }

  const rows = $derived.by(() => {
    const base = $podsByStatus[activeSegment];
    const q = filter.trim().toLowerCase();
    return q ? base.filter((p) => matches(p, q)) : base;
  });

  const tableState = $derived.by<TableState>(() => {
    switch ($loadState.status) {
      case 'error':
        return 'error';
      case 'forbidden':
        return 'forbidden';
      case 'ready':
        return 'ready';
      default:
        return 'loading';
    }
  });

  // Verdicts are recomputed only when a pod changes, so cache per object.
  const healthCache = new WeakMap<PodInfo, HealthResult>();
  function health(p: PodInfo): HealthResult {
    let h = healthCache.get(p);
    if (!h) {
      h = toDisplay(p.verdict);
      healthCache.set(p, h);
    }
    return h;
  }

  const rowKey = (p: PodInfo) => `${p.namespace}/${p.name}`;

  const columns: Column<PodInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%' },
    {
      id: 'status',
      label: 'Status',
      width: '18%',
      sortKey: (p) => health(p).rank,
      cell: statusCell,
      title: (p) => `${p.verdict.reason} (${p.verdict.ruleId})`,
    },
    { id: 'ready', label: 'Ready', sortKey: 'ready', numeric: true, width: '72px' },
    { id: 'restarts', label: 'Restarts', sortKey: 'restarts', numeric: true, width: '88px', cell: restartsCell },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
    { id: 'node', label: 'Node', sortKey: 'node', width: '16%', hideBelow: 1100, mono: true },
  ];

  // ---- Failing-count live region (spec 8): debounced 5s, polite ----
  let liveText = $state('');
  let liveTimer: ReturnType<typeof setTimeout> | null = null;
  let lastAnnounced: number | null = null;
  // A derived number only re-runs the effect when the count itself changes,
  // so a busy watch stream can't keep pushing the announcement back.
  const failingCount = $derived($podsByStatus.failed.length);
  const loaded = $derived($loadState.status === 'ready');
  $effect(() => {
    const n = failingCount;
    if (!loaded) return;
    if (liveTimer) clearTimeout(liveTimer);
    liveTimer = setTimeout(() => {
      if (n === lastAnnounced) return;
      lastAnnounced = n;
      liveText = n === 0 ? 'No failing pods' : `${n} failing ${n === 1 ? 'pod' : 'pods'}`;
    }, 5000);
  });

  onDestroy(() => {
    if (liveTimer) clearTimeout(liveTimer);
    stopPodWatch();
  });

  // Starts the watch on mount, and again on namespace, context or refresh.
  $effect(() => {
    const ctx = $currentContext;
    void $refreshTrigger;
    if (!ctx) return;
    startPodWatch($selectedNamespace);
  });

  function openDetail(pod: PodInfo, tab?: string) {
    invoke('open_resource_detail', {
      resourceType: 'pod',
      name: pod.name,
      namespace: pod.namespace,
      context: $currentContext,
      ...(tab ? { tab } : {}),
    }).catch((e) => console.error('Failed to open pod detail:', e));
  }

  const openLogs = (p: PodInfo) => openDetail(p, 'logs');
  const exec = (p: PodInfo) => openTerminalWindow(p.namespace, p.name);
</script>

{#snippet statusCell(p: PodInfo)}
  {@const h = health(p)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

{#snippet restartsCell(p: PodInfo)}
  {#if p.restarts >= 20}
    <span class="inline-flex items-center gap-1 text-danger" title="{p.restarts} restarts">
      <StatusIcon health="warning" colored={false} />{p.restarts}
    </span>
  {:else if p.restarts >= 5}
    <span class="inline-flex items-center gap-1 text-warning" title="{p.restarts} restarts">
      <StatusIcon health="warning" colored={false} />{p.restarts}
    </span>
  {:else}
    {p.restarts}
  {/if}
{/snippet}

{#snippet rowActions(p: PodInfo)}
  <IconButton label="Logs" shortcut="L" size="sm" onclick={() => openLogs(p)}>
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
      <path d="M3 4h10M3 8h10M3 12h6" />
    </svg>
  </IconButton>
  <IconButton label="Exec" shortcut="E" size="sm" onclick={() => exec(p)}>
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M3 5l3 3-3 3M8 11h5" />
    </svg>
  </IconButton>
{/snippet}

{#snippet emptyActions()}
  {#if $selectedNamespace}
    <Button onclick={() => selectedNamespace.set(null)}>All namespaces</Button>
  {/if}
{/snippet}

<div class="h-full flex flex-col overflow-hidden">
  <PageHeader
    title="Pods"
    count={$podsByStatus.all.length}
    {segments}
    {activeSegment}
    onSegment={(id) => (activeSegment = id as SegmentId)}
    bind:filter
    filterPlaceholder="Filter pods…"
    viewId="pods"
    loading={$loadState.status === 'loading' && $podsByStatus.all.length > 0}
  />

  <DataTable
    {rows}
    {columns}
    {rowKey}
    {health}
    defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
    state={tableState}
    error={$loadState.error}
    filtered={rows.length === 0 && $podsByStatus.all.length > 0}
    filterText={filter}
    itemLabel="pods"
    ariaLabel="Pods"
    onOpen={(p) => openDetail(p)}
    onClearFilter={() => {
      filter = '';
      activeSegment = 'all';
    }}
    onRetry={() => startPodWatch($selectedNamespace)}
    onRefresh={() => startPodWatch($selectedNamespace)}
    copyText={(p) => p.name}
    emptyState={{
      title: $selectedNamespace ? `No pods in ${$selectedNamespace}` : 'No pods',
      hint: 'Switch namespace or check the context.',
    }}
    {emptyActions}
    {rowActions}
    shortcuts={{ l: openLogs, e: exec }}
  />

  <div class="sr-only" aria-live="polite" aria-atomic="true">{liveText}</div>
</div>
