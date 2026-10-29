<script lang="ts" generics="T">
  // One list page in the Pods style (spec 5.3 + 5.4): PageHeader with the
  // health segments and filter, then DataTable. Every list view except Pods
  // is a column config on this.
  //
  // Data comes from a ListSource (stores/kubernetes.ts). A polled ListStore
  // is loaded here: on mount, context, namespace (namespaced lists only),
  // refresh and `extra` changes, plus a 10s poll tick that skips while a
  // load is in flight. Watched lists pass `autoload={false}` and run their
  // watch in the view.
  //
  // Rows are tagged with the context they came from. A row from another
  // context is never opened or acted on (the source clears on a switch, so
  // this is the backstop), and the detail window gets the row's context,
  // not whatever is current by the time the click lands.
  //
  // Lists never delete, scale or restart (CLAUDE.md, Architecture Note 7):
  // DataTable turns `d` into "open detail"; row actions are non-destructive.
  import type { Snippet } from 'svelte';
  import { onMount, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import PageHeader, { type Segment } from './PageHeader.svelte';
  import DataTable from './DataTable.svelte';
  import Button from './Button.svelte';
  import {
    cellText,
    filterRows,
    healthCounts,
    tableStateOf,
    SEGMENT_ORDER,
    type Column,
    type FilterField,
    type TableSort,
    type TableState,
  } from './data-table';
  import type { Health, HealthResult } from '../../utils/health';
  import { LIST_POLL_MS } from '../../utils/poll';
  import { toast } from '../../stores/toast';
  import {
    currentContext,
    selectedNamespace,
    refreshTrigger,
    type ListSource,
    type ListStore,
  } from '../../stores/kubernetes';

  let {
    title,
    viewId,
    itemLabel,
    source,
    rows: rowsOverride,
    autoload,
    extra,
    enabled = true,
    namespaced: namespacedOverride,
    columns,
    rowKey,
    filterFields,
    health,
    segmentLabels,
    defaultSort,
    resourceType,
    subtitle,
    onOpen,
    onRefresh,
    copyText,
    emptyHint,
    actions,
    rowActions,
    shortcuts,
    minWidth = '360px',
    keyboard = true,
    rowFilter,
    onClearFilter,
    deleteHint,
  }: {
    title: string;
    /** Keys the session-remembered filter. */
    viewId: string;
    /** Plural noun for state messages: "deployments". */
    itemLabel: string;
    source: ListSource<T> | ListStore<T>;
    /** Rows to show instead of the source's (e.g. nodes merged with metrics). */
    rows?: T[];
    /** Load the source here (default: true for a ListStore). */
    autoload?: boolean;
    /** Extra command args, e.g. the CRD target. Changing it reloads. */
    extra?: Record<string, unknown>;
    /** False skips loading (the CRD browser before a CRD is picked). */
    enabled?: boolean;
    /** Override the source's scope (cluster-scoped custom resources). */
    namespaced?: boolean;
    columns: Column<T>[];
    rowKey: (r: T) => string;
    /** Fields the filter searches (arrays joined). Default: every column's text. */
    filterFields?: FilterField<T>[];
    health?: (r: T) => HealthResult;
    /** Status segments by health, with labels. Only listed states get one; "All" is first. */
    segmentLabels?: Partial<Record<Health, string>>;
    defaultSort?: TableSort;
    /** Detail window type, per list or per row. Ignored when `onOpen` is set. */
    resourceType?: string | ((r: T) => string);
    subtitle?: string;
    /** Click / Enter. Defaults to opening the detail window. */
    onOpen?: (r: T) => void;
    /** `r` and Retry. Defaults to reloading the source. */
    onRefresh?: () => void;
    /** `y` copies this (default: the row's name). */
    copyText?: (r: T) => string;
    emptyHint?: string;
    /** Non-destructive header actions (Create, filters). */
    actions?: Snippet;
    rowActions?: Snippet<[T]>;
    shortcuts?: Record<string, (r: T) => void>;
    minWidth?: string;
    /** False while the view shows its own modal (pauses row keys and j/k). */
    keyboard?: boolean;
    /** An extra view filter (e.g. the Events reason select). */
    rowFilter?: (r: T) => boolean;
    /** Also called by "Clear filter", to reset the view's own filters. */
    onClearFilter?: () => void;
    /** Toast for `d` (default: "Delete from the detail window"). */
    deleteHint?: string;
  } = $props();

  const itemsStore = $derived(source.items);
  const stateStore = $derived(source.state);
  const contextStore = $derived(source.context);

  const items = $derived(rowsOverride ?? $itemsStore);
  const isNamespaced = $derived(namespacedOverride ?? source.namespaced);
  const loads = $derived((autoload ?? 'load' in source) && 'load' in source);

  function reload(poll = false) {
    if (!enabled || !('load' in source)) return;
    (source as ListStore<T>).load(isNamespaced ? $selectedNamespace : null, extra ?? {}, {
      poll,
      namespaced: isNamespaced,
    });
  }

  // Mount, context, refresh, a new `extra`, and the namespace (namespaced lists only).
  $effect(() => {
    if (!loads) return;
    void $currentContext;
    void $refreshTrigger;
    void extra;
    void enabled;
    if (isNamespaced) void $selectedNamespace;
    untrack(() => reload());
  });

  onMount(() => {
    const id = setInterval(() => {
      if (loads) reload(true);
    }, LIST_POLL_MS);
    return () => clearInterval(id);
  });

  const refresh = () => (onRefresh ? onRefresh() : reload());

  // ---- Segments + filters ----
  let filter = $state('');
  let activeSegment = $state<string>('all');

  const counts = $derived(health ? healthCounts(items, health) : null);

  const segments = $derived.by<Segment[] | undefined>(() => {
    if (!segmentLabels || !counts) return undefined;
    const out: Segment[] = [{ id: 'all', label: 'All', count: items.length }];
    for (const h of SEGMENT_ORDER) {
      const label = segmentLabels[h];
      if (label) out.push({ id: h, label, count: counts[h], health: h });
    }
    return out;
  });

  // A segment that empties out (the last failing row recovered) falls back to All.
  $effect(() => {
    if (activeSegment !== 'all' && counts && counts[activeSegment as Health] === 0) activeSegment = 'all';
  });

  const textFields = $derived<FilterField<T>[]>(filterFields ?? columns.map((c) => (r: T) => cellText(r, c)));

  const rows = $derived.by(() => {
    let base =
      activeSegment === 'all' || !health ? items : items.filter((r) => health(r).health === activeSegment);
    if (rowFilter) base = base.filter(rowFilter);
    return filterRows(base, filter, textFields);
  });

  const tableState = $derived<TableState>(
    !enabled && ($stateStore.status === 'idle' || $stateStore.status === 'loading') ? 'ready' : tableStateOf($stateStore.status)
  );

  // ---- Context guard ----
  // Rows loaded from another context (or not yet tagged) must not be acted on.
  const rowsContext = $derived($contextStore);
  const stale = () => items.length > 0 && rowsContext !== $currentContext;

  function guarded(fn: (r: T) => void): (r: T) => void {
    return (r) => {
      if (stale()) {
        toast('Still loading this cluster. Try again in a moment.');
        return;
      }
      fn(r);
    };
  }

  const nameOf = (r: T) => (r as { name: string }).name;

  function openDetail(r: T) {
    const row = r as { name: string; namespace?: string };
    const type = typeof resourceType === 'function' ? resourceType(r) : resourceType;
    invoke('open_resource_detail', {
      resourceType: type,
      name: row.name,
      namespace: isNamespaced ? (row.namespace ?? '') : '',
      context: rowsContext || $currentContext,
    }).catch((e) => console.error(`Failed to open ${type} detail:`, e));
  }

  const open = $derived(guarded(onOpen ?? openDetail));
  const guardedShortcuts = $derived(
    shortcuts ? Object.fromEntries(Object.entries(shortcuts).map(([k, fn]) => [k, guarded(fn)])) : undefined
  );
</script>

{#snippet emptyActions()}
  {#if isNamespaced && $selectedNamespace}
    <Button onclick={() => selectedNamespace.set(null)}>All namespaces</Button>
  {/if}
{/snippet}

<div class="h-full flex flex-col overflow-hidden">
  <PageHeader
    {title}
    count={enabled ? items.length : undefined}
    {subtitle}
    {segments}
    {activeSegment}
    onSegment={(id) => (activeSegment = id)}
    bind:filter
    filterPlaceholder={`Filter ${itemLabel}…`}
    {viewId}
    loading={$stateStore.status === 'loading' && items.length > 0}
    {actions}
  />

  <DataTable
    {rows}
    {columns}
    {rowKey}
    {health}
    {defaultSort}
    state={tableState}
    error={$stateStore.error}
    filtered={rows.length === 0 && items.length > 0}
    filterText={filter}
    {itemLabel}
    ariaLabel={title}
    onOpen={open}
    onClearFilter={() => {
      filter = '';
      activeSegment = 'all';
      onClearFilter?.();
    }}
    onRetry={refresh}
    onRefresh={refresh}
    copyText={copyText ?? nameOf}
    emptyState={{
      title: isNamespaced && $selectedNamespace ? `No ${itemLabel} in ${$selectedNamespace}` : `No ${itemLabel}`,
      hint: emptyHint ?? (isNamespaced ? 'Switch namespace or check the context.' : 'Check the context.'),
    }}
    {emptyActions}
    {rowActions}
    shortcuts={guardedShortcuts}
    {minWidth}
    {keyboard}
    {deleteHint}
  />
</div>
