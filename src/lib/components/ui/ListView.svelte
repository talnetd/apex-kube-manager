<script lang="ts" generics="T">
  // One list page in the Pods style (spec 5.3 + 5.4): PageHeader with the
  // health segments and filter, then DataTable. Views own their data (watch
  // or poll) and columns; this owns the filter, segments, table states, the
  // "All namespaces" empty action and opening the detail window.
  //
  // Lists never delete, scale or restart (CLAUDE.md, Architecture Note 7):
  // DataTable turns `d` into "open detail", and row actions are
  // non-destructive only.
  import type { Snippet } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import PageHeader, { type Segment } from './PageHeader.svelte';
  import DataTable from './DataTable.svelte';
  import Button from './Button.svelte';
  import {
    filterRows,
    healthCounts,
    tableStateOf,
    SEGMENT_ORDER,
    type Column,
    type FilterField,
    type TableSort,
  } from './data-table';
  import type { Health, HealthResult } from '../../utils/health';
  import { currentContext, selectedNamespace, type ResourceState } from '../../stores/kubernetes';

  let {
    title,
    viewId,
    itemLabel,
    items,
    loadState,
    columns,
    rowKey,
    filterFields,
    health,
    segmentLabels,
    defaultSort,
    resourceType,
    namespaced = true,
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
  }: {
    title: string;
    /** Keys the session-remembered filter. */
    viewId: string;
    /** Plural noun for state messages: "deployments". */
    itemLabel: string;
    items: T[];
    loadState: ResourceState;
    columns: Column<T>[];
    rowKey: (r: T) => string;
    /** Fields the filter input searches (arrays are joined). */
    filterFields: FilterField<T>[];
    health?: (r: T) => HealthResult;
    /**
     * Status segments to show, by health, with their labels. Only the
     * states listed get a segment; "All" is always first.
     */
    segmentLabels?: Partial<Record<Health, string>>;
    defaultSort?: TableSort;
    /** Detail window type, e.g. 'deployment'. Ignored when `onOpen` is set. */
    resourceType?: string;
    namespaced?: boolean;
    subtitle?: string;
    /** Click / Enter. Defaults to opening the detail window. */
    onOpen?: (r: T) => void;
    /** `r` and the error state's Retry. */
    onRefresh: () => void;
    /** `y` copies this (default: the row's name). */
    copyText?: (r: T) => string;
    emptyHint?: string;
    /** Non-destructive header actions (Create, filters). */
    actions?: Snippet;
    rowActions?: Snippet<[T]>;
    shortcuts?: Record<string, (r: T) => void>;
    minWidth?: string;
    /** False while the view shows its own modal. */
    keyboard?: boolean;
    /** An extra view filter (e.g. the Events reason select), applied with the text filter. */
    rowFilter?: (r: T) => boolean;
    /** Also called by "Clear filter", to reset the view's own filters. */
    onClearFilter?: () => void;
  } = $props();

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

  const rows = $derived.by(() => {
    let base =
      activeSegment === 'all' || !health ? items : items.filter((r) => health(r).health === activeSegment);
    if (rowFilter) base = base.filter(rowFilter);
    return filterRows(base, filter, filterFields);
  });

  const nameOf = (r: T) => (r as { name: string }).name;

  function openDetail(r: T) {
    const row = r as { name: string; namespace?: string };
    invoke('open_resource_detail', {
      resourceType,
      name: row.name,
      namespace: namespaced ? (row.namespace ?? '') : '',
      context: $currentContext,
    }).catch((e) => console.error(`Failed to open ${resourceType} detail:`, e));
  }
</script>

{#snippet emptyActions()}
  {#if namespaced && $selectedNamespace}
    <Button onclick={() => selectedNamespace.set(null)}>All namespaces</Button>
  {/if}
{/snippet}

<div class="h-full flex flex-col overflow-hidden">
  <PageHeader
    {title}
    count={items.length}
    {subtitle}
    {segments}
    {activeSegment}
    onSegment={(id) => (activeSegment = id)}
    bind:filter
    filterPlaceholder={`Filter ${itemLabel}…`}
    {viewId}
    loading={loadState.status === 'loading' && items.length > 0}
    {actions}
  />

  <DataTable
    {rows}
    {columns}
    {rowKey}
    {health}
    {defaultSort}
    state={tableStateOf(loadState.status)}
    error={loadState.error}
    filtered={rows.length === 0 && items.length > 0}
    filterText={filter}
    {itemLabel}
    ariaLabel={title}
    onOpen={onOpen ?? openDetail}
    onClearFilter={() => {
      filter = '';
      activeSegment = 'all';
      onClearFilter?.();
    }}
    onRetry={onRefresh}
    {onRefresh}
    copyText={copyText ?? nameOf}
    emptyState={{
      title: namespaced && $selectedNamespace ? `No ${itemLabel} in ${$selectedNamespace}` : `No ${itemLabel}`,
      hint: emptyHint ?? (namespaced ? 'Switch namespace or check the context.' : 'Check the context.'),
    }}
    {emptyActions}
    {rowActions}
    {shortcuts}
    {minWidth}
    {keyboard}
  />
</div>
