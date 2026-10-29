<script lang="ts" generics="T">
  // Shared list view for the extra resource types (spec 12): PageHeader +
  // DataTable in the Pods style. The page supplies columns; this owns
  // loading (10s poll, like the other lists), the filter, the namespace
  // and context reactions, and opening the detail window. Lists never
  // delete (CLAUDE.md, Architecture Note 7).
  import type { Snippet } from 'svelte';
  import { onMount, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import PageHeader from '../ui/PageHeader.svelte';
  import DataTable from '../ui/DataTable.svelte';
  import Button from '../ui/Button.svelte';
  import { cellText, type Column, type TableSort, type TableState } from '../ui/data-table';
  import type { HealthResult } from '../../utils/health';
  import { currentContext, selectedNamespace, refreshTrigger } from '../../stores/kubernetes';
  import type { ListStore } from '../../stores/resources';

  let {
    title,
    itemLabel,
    viewId,
    store,
    columns,
    rowKey,
    resourceType,
    health,
    defaultSort,
    extra,
    subtitle,
    actions,
    emptyHint,
    rowActions,
    onOpen,
    enabled = true,
  }: {
    title: string;
    /** Plural noun: "roles". */
    itemLabel: string;
    viewId: string;
    store: ListStore<T>;
    columns: Column<T>[];
    rowKey: (r: T) => string;
    /** Detail window type id (string, or per row for custom resources). */
    resourceType: string | ((r: T) => string);
    health?: (r: T) => HealthResult;
    defaultSort?: TableSort;
    /** Extra command args, e.g. the CRD target. Changing it reloads. */
    extra?: Record<string, unknown>;
    subtitle?: string;
    actions?: Snippet;
    emptyHint?: string;
    /** Non-destructive row actions, shown on hover/selection. */
    rowActions?: Snippet<[T]>;
    /** Override what Enter/click does (default: open the detail window). */
    onOpen?: (r: T) => void;
    /** False skips loading (the CRD browser before a CRD is picked). */
    enabled?: boolean;
  } = $props();

  const items = $derived(store.items);
  const loadState = $derived(store.state);
  let filter = $state('');

  const reload = () => {
    if (enabled) store.load(store.namespaced ? $selectedNamespace : null, extra);
  };

  // Mount, namespace, context, refresh or a new `extra` all reload.
  $effect(() => {
    void $currentContext;
    void $selectedNamespace;
    void $refreshTrigger;
    void extra;
    void enabled;
    untrack(reload);
  });

  onMount(() => {
    const id = setInterval(reload, 10000);
    return () => clearInterval(id);
  });

  const rows = $derived.by(() => {
    const q = filter.trim().toLowerCase();
    if (!q) return $items;
    return $items.filter((r) => columns.some((c) => cellText(r, c).toLowerCase().includes(q)));
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
        return enabled ? 'loading' : 'ready';
    }
  });

  function openDetail(r: T) {
    const row = r as { name: string; namespace?: string };
    const type = typeof resourceType === 'function' ? resourceType(r) : resourceType;
    invoke('open_resource_detail', {
      resourceType: type,
      name: row.name,
      namespace: row.namespace ?? '',
      context: $currentContext,
    }).catch((e) => console.error(`Failed to open ${type} detail:`, e));
  }
</script>

{#snippet emptyActions()}
  {#if store.namespaced && $selectedNamespace}
    <Button onclick={() => selectedNamespace.set(null)}>All namespaces</Button>
  {/if}
{/snippet}

<div class="h-full flex flex-col overflow-hidden">
  <PageHeader
    {title}
    count={enabled ? $items.length : undefined}
    {subtitle}
    bind:filter
    filterPlaceholder={`Filter ${itemLabel}…`}
    {viewId}
    loading={$loadState.status === 'loading' && $items.length > 0}
    {actions}
  />
  <DataTable
    {rows}
    {columns}
    {rowKey}
    {health}
    {defaultSort}
    state={tableState}
    error={$loadState.error}
    filtered={rows.length === 0 && $items.length > 0}
    filterText={filter}
    {itemLabel}
    ariaLabel={title}
    onOpen={onOpen ?? openDetail}
    {rowActions}
    onClearFilter={() => (filter = '')}
    onRetry={reload}
    onRefresh={reload}
    copyText={(r) => (r as { name: string }).name}
    emptyState={{
      title: store.namespaced && $selectedNamespace ? `No ${itemLabel} in ${$selectedNamespace}` : `No ${itemLabel}`,
      hint: emptyHint,
    }}
    {emptyActions}
  />
</div>
