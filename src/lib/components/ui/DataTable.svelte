<script lang="ts" module>
  // The keyboard store is global; only the most recently mounted table owns
  // it, so an outgoing view's teardown can't zero the incoming one.
  let navOwner = 0;
  let nextTableId = 1;
</script>

<script lang="ts" generics="T">
  // Shared list table (spec 5.4). Owns keyboard selection (totalRows,
  // selectedRowIndex, j/k/gg/G handled globally in App.svelte), selection
  // anchoring by row key, scroll-into-view, Enter/y/r/d and view shortcuts,
  // sorting, sticky header, and the loading / empty / filtered / error /
  // forbidden states. List views supply rows and columns only.
  //
  // Destructive actions never live here (CLAUDE.md, Architecture Note 7):
  // `d` opens the detail window instead, with a hint toast.
  import type { Snippet } from 'svelte';
  import { onMount, onDestroy, untrack } from 'svelte';
  import { get } from 'svelte/store';
  import {
    selectedRowIndex,
    keyboardNavActive,
    totalRows,
    resetNavigation,
  } from '../../stores/keyboard';
  import { density } from '../../stores/density';
  import { toast } from '../../stores/toast';
  import { anchorSelection } from '../../utils/selection';
  import { isTypingTarget } from '../../utils/keys';
  import type { HealthResult } from '../../utils/health';
  import EmptyState from './EmptyState.svelte';
  import Button from './Button.svelte';
  import {
    sortRows,
    nextSort,
    cellText,
    type Column,
    type TableSort,
    type TableState,
  } from './data-table';

  let {
    rows,
    columns,
    rowKey,
    health,
    defaultSort,
    state: tableState,
    error,
    filtered = false,
    filterText = '',
    itemLabel = 'items',
    onOpen,
    onClearFilter,
    onRetry,
    onRefresh,
    copyText,
    emptyState,
    emptyActions,
    rowActions,
    shortcuts,
    ariaLabel,
  }: {
    rows: T[];
    columns: Column<T>[];
    /** e.g. `${ns}/${name}`; anchors the selection across re-sorts. */
    rowKey: (r: T) => string;
    /** Drives the row rail and the synthetic 'health' sort. */
    health?: (r: T) => HealthResult;
    defaultSort?: TableSort;
    state: TableState;
    error?: string;
    /** True when a filter or segment hides rows (filtered-empty state). */
    filtered?: boolean;
    filterText?: string;
    /** Plural noun for state messages: "pods". */
    itemLabel?: string;
    onOpen: (r: T) => void;
    onClearFilter?: () => void;
    onRetry?: () => void;
    /** `r` key. */
    onRefresh?: () => void;
    /** `y` key copies this (default: rowKey). */
    copyText?: (r: T) => string;
    emptyState?: { title: string; hint?: string };
    emptyActions?: Snippet;
    /** Non-destructive only (logs, exec). Shown on hover / selection. */
    rowActions?: Snippet<[T]>;
    /** View-specific keys, e.g. { l: openLogs, e: exec }. */
    shortcuts?: Record<string, (r: T) => void>;
    ariaLabel?: string;
  } = $props();

  // A "default" sort the header cycle returns to.
  let sort = $state<TableSort | null>(untrack(() => defaultSort ?? null));

  let viewportWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 1280);
  const visibleColumns = $derived(columns.filter((c) => !c.hideBelow || viewportWidth >= c.hideBelow));

  const sorted = $derived(sortRows(rows, columns, sort, rowKey, health));

  let rowEls: HTMLTableRowElement[] = $state([]);

  // ---- Selection anchoring (moved from NodeList, spec 5.4) ----
  let anchoredKey: string | null = null;
  let lastIndex = -1;

  // When the user moves the selection, re-anchor to the row now under it.
  $effect(() => {
    const idx = $selectedRowIndex;
    untrack(() => {
      lastIndex = idx;
      anchoredKey = idx >= 0 && idx < sorted.length ? rowKey(sorted[idx]) : null;
    });
  });

  // When rows change (watch update, re-sort), follow the anchored row.
  $effect(() => {
    const items = sorted;
    untrack(() => {
      totalRows.set(items.length);
      const keys = items.map(rowKey);
      const next = anchorSelection(anchoredKey, lastIndex, keys);
      anchoredKey = next >= 0 ? keys[next] : null;
      lastIndex = next;
      if (next !== get(selectedRowIndex)) selectedRowIndex.set(next);
    });
  });

  // Scroll the keyboard-selected row into view; not smooth in compact.
  $effect(() => {
    const idx = $selectedRowIndex;
    if (!$keyboardNavActive || idx < 0) return;
    const el = rowEls[idx];
    if (el) el.scrollIntoView({ block: 'nearest', behavior: get(density) === 'compact' ? 'auto' : 'smooth' });
  });

  const selectedRow = $derived(
    $selectedRowIndex >= 0 && $selectedRowIndex < sorted.length ? sorted[$selectedRowIndex] : null
  );

  function copy(r: T) {
    const text = copyText ? copyText(r) : rowKey(r);
    navigator.clipboard?.writeText(text).then(
      () => toast(`Copied ${text}`),
      () => toast("Couldn't copy to clipboard", { health: 'failed' })
    );
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.defaultPrevented || e.metaKey || e.ctrlKey || e.altKey) return;
    if (isTypingTarget(document.activeElement)) return;

    if (e.key === 'r' && onRefresh) {
      e.preventDefault();
      onRefresh();
      return;
    }

    const r = selectedRow;
    if (!r) return;

    if (e.key === 'Enter') {
      // A focused button inside the page handles Enter itself.
      if ((document.activeElement as HTMLElement | null)?.tagName === 'BUTTON') return;
      e.preventDefault();
      onOpen(r);
      return;
    }
    if (e.key === 'y') {
      e.preventDefault();
      copy(r);
      return;
    }
    const custom = shortcuts?.[e.key];
    if (custom) {
      e.preventDefault();
      custom(r);
      return;
    }
    if (e.key === 'd') {
      // Muscle memory from the old list delete: open the detail window instead.
      e.preventDefault();
      onOpen(r);
      toast('Delete from the detail window');
    }
  }

  // Fresh selection per view. Done at init (not in onMount) so it runs
  // before the effects above publish totalRows.
  const tableId = nextTableId++;
  navOwner = tableId;
  resetNavigation();

  onMount(() => {
    totalRows.set(untrack(() => sorted.length));
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
    if (navOwner === tableId) {
      navOwner = 0;
      resetNavigation();
    }
  });

  function clickRow(i: number, r: T) {
    selectedRowIndex.set(i);
    onOpen(r);
  }

  function ariaSort(colId: string): 'ascending' | 'descending' | 'none' {
    if (!sort || sort.id !== colId) return 'none';
    return sort.dir === 'asc' ? 'ascending' : 'descending';
  }

  // Row rails are drawn as inset shadows on the edge cells (box-shadow on <tr>
  // is unreliable in WebKit). Decorative: the status pill carries the meaning.
  function rails(r: T, selected: boolean, first: boolean, last: boolean): string | undefined {
    if (!first && !last) return undefined;
    const h = health?.(r).health;
    const problem =
      h === 'failed' ? 'var(--ui-danger-solid)' : h === 'warning' ? 'var(--ui-warning-solid)' : null;
    const shadows: string[] = [];
    if (first) {
      if (selected) shadows.push('inset 2px 0 0 var(--ui-accent)');
      else if (problem) shadows.push(`inset 2px 0 0 ${problem}`);
    }
    // Failed + selected: the accent wins on the left, the problem keeps a right rail.
    if (last && selected && problem) shadows.push(`inset -2px 0 0 ${problem}`);
    return shadows.length ? shadows.join(', ') : undefined;
  }

  const SKELETON_WIDTHS = ['40%', '15%', '12%', '8%', '10%', '14%'];
  const showSkeleton = $derived(tableState === 'loading' && rows.length === 0);
  const showRows = $derived(sorted.length > 0 && tableState !== 'error' && tableState !== 'forbidden');
</script>

<svelte:window bind:innerWidth={viewportWidth} />

<div class="relative flex-1 min-h-0 overflow-auto bg-app">
  <table
    class="dt w-full table-fixed border-separate border-spacing-0"
    role="grid"
    tabindex="0"
    aria-label={ariaLabel ?? itemLabel}
    aria-rowcount={sorted.length + 1}
    aria-busy={tableState === 'loading' ? 'true' : undefined}
  >
    <colgroup>
      {#each visibleColumns as col (col.id)}
        <col style:width={col.width} />
      {/each}
      {#if rowActions}<col style:width="72px" />{/if}
    </colgroup>
    <thead>
      <tr>
        {#each visibleColumns as col (col.id)}
          {@const right = col.numeric || col.align === 'right'}
          <th
            scope="col"
            aria-sort={col.sortKey !== undefined ? ariaSort(col.id) : undefined}
            class="sticky top-0 z-10 bg-subtle border-b border-line h-(--header-row-h) px-(--cell-px) text-2xs font-medium uppercase tracking-[0.04em] text-muted whitespace-nowrap {right ? 'text-right' : 'text-left'}"
          >
            {#if col.sortKey !== undefined}
              <button
                type="button"
                class="inline-flex items-center gap-1 max-w-full uppercase tracking-[0.04em] hover:text-primary {sort?.id === col.id ? 'text-secondary' : ''} {right ? 'flex-row-reverse' : ''}"
                onclick={() => (sort = nextSort(sort, col.id, defaultSort ?? null))}
              >
                <span class="truncate">{col.label}</span>
                {#if sort?.id === col.id}
                  <svg width="8" height="8" viewBox="0 0 8 8" fill="currentColor" aria-hidden="true">
                    {#if sort.dir === 'asc'}<path d="M4 1.5 7 6H1z" />{:else}<path d="M4 6.5 1 2h6z" />{/if}
                  </svg>
                {/if}
              </button>
            {:else}
              <span class="truncate">{col.label}</span>
            {/if}
          </th>
        {/each}
        {#if rowActions}<th scope="col" class="sticky top-0 z-10 bg-subtle border-b border-line h-(--header-row-h)"><span class="sr-only">Actions</span></th>{/if}
      </tr>
    </thead>

    <tbody>
      {#if showSkeleton}
        {#each Array(8) as _, i}
          <tr class="h-(--row-h)" aria-hidden="true">
            {#each visibleColumns as col, c (col.id)}
              <td class="border-b border-divider px-(--cell-px)">
                <div
                  class="skeleton h-2.5 rounded-sm bg-hover {col.numeric || col.align === 'right' ? 'ml-auto' : ''}"
                  style:width={SKELETON_WIDTHS[(c + i) % SKELETON_WIDTHS.length]}
                  style:max-width="100%"
                ></div>
              </td>
            {/each}
            {#if rowActions}<td class="border-b border-divider"></td>{/if}
          </tr>
        {/each}
      {:else if showRows}
        {#each sorted as row, i (rowKey(row))}
          {@const selected = i === $selectedRowIndex}
          <tr
            bind:this={rowEls[i]}
            class="row h-(--row-h) cursor-pointer {selected ? 'selected bg-selected' : 'hover:bg-hover'}"
            aria-selected={selected}
            aria-rowindex={i + 2}
            data-key={rowKey(row)}
            onclick={() => clickRow(i, row)}
          >
            {#each visibleColumns as col, c (col.id)}
              {@const right = col.numeric || col.align === 'right'}
              {@const text = col.cell ? undefined : cellText(row, col)}
              <td
                class="border-b border-divider px-(--cell-px) whitespace-nowrap overflow-hidden {right ? 'text-right' : 'text-left'} {col.numeric ? 'tabular-nums' : ''} {col.mono ? 'font-mono' : ''} {col.primary ? 'text-primary font-medium' : 'text-secondary'}"
                style="font-size: var(--cell-font)"
                style:box-shadow={rails(row, selected, c === 0, !rowActions && c === visibleColumns.length - 1)}
              >
                <div class="truncate {col.primary ? 'name' : ''}" title={col.title ? col.title(row) : text !== '–' ? text : undefined}>
                  {#if col.cell}{@render col.cell(row)}{:else}{text}{/if}
                </div>
              </td>
            {/each}
            {#if rowActions}
              <td
                class="border-b border-divider px-1 text-right"
                style:box-shadow={rails(row, selected, false, true)}
                onclick={(e) => e.stopPropagation()}
              >
                <div class="row-actions inline-flex items-center gap-0.5">{@render rowActions(row)}</div>
              </td>
            {/if}
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>

  {#if !showSkeleton && !showRows}
    {#if tableState === 'error'}
      <EmptyState variant="error" title={`Couldn't load ${itemLabel}`} detail={error}>
        {#snippet actions()}
          {#if onRetry}<Button variant="primary" onclick={onRetry}>Retry</Button>{/if}
          {#if error}
            <Button
              variant="secondary"
              onclick={() => navigator.clipboard?.writeText(error ?? '').then(() => toast('Error copied'))}
            >Copy error</Button>
          {/if}
        {/snippet}
      </EmptyState>
    {:else if tableState === 'forbidden'}
      <EmptyState
        variant="forbidden"
        title={emptyState?.title ?? `No access to ${itemLabel}`}
        hint="Your kube user can't list this resource."
      />
    {:else if filtered}
      <EmptyState
        variant="filtered"
        title={filterText ? `No ${itemLabel} match "${filterText}"` : `No ${itemLabel} match this filter`}
      >
        {#snippet actions()}
          {#if onClearFilter}<Button onclick={onClearFilter}>Clear filter</Button>{/if}
        {/snippet}
      </EmptyState>
    {:else}
      <EmptyState
        variant="empty"
        title={emptyState?.title ?? `No ${itemLabel}`}
        hint={emptyState?.hint}
        actions={emptyActions}
      />
    {/if}
  {/if}
</div>

<style>
  .dt:focus-visible {
    outline: none;
  }
  .dt:focus-visible :global(tr.selected) {
    outline: 2px solid var(--ui-accent);
    outline-offset: -2px;
  }
  .row .row-actions {
    opacity: 0;
    transition: opacity 100ms;
  }
  .row:hover .row-actions,
  .row.selected .row-actions,
  .row:focus-within .row-actions {
    opacity: 1;
  }
  .row:hover .name {
    text-decoration: underline;
    text-underline-offset: 2px;
    text-decoration-color: var(--ui-line-strong);
  }
  .skeleton {
    animation: skeleton-pulse 1.2s ease-in-out infinite alternate;
  }
  @keyframes skeleton-pulse {
    from {
      opacity: 0.5;
    }
    to {
      opacity: 1;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .skeleton {
      animation: none;
    }
  }
</style>
