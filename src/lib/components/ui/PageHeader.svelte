<script lang="ts" module>
  import type { Health } from '../../utils/health';

  export interface Segment {
    id: string;
    label: string;
    count: number;
    health?: Health;
  }

  // Filter text per view, kept for the session (spec 5.3): switching
  // Pods -> Nodes -> Pods keeps it.
  const sessionFilters = new Map<string, string>();
  export function rememberedFilter(viewId: string): string {
    return sessionFilters.get(viewId) ?? '';
  }
</script>

<script lang="ts">
  // Page header (spec 5.3): title + count, segmented status filter, an
  // always-visible filter input, and a slot for non-destructive actions.
  import type { Snippet } from 'svelte';
  import { onMount } from 'svelte';
  import StatusIcon from './StatusIcon.svelte';
  import Kbd from './Kbd.svelte';
  import { isTypingTarget } from '../../utils/keys';

  let {
    title,
    count,
    subtitle,
    segments,
    activeSegment,
    onSegment,
    filter = $bindable(''),
    filterPlaceholder = 'Filter…',
    viewId,
    loading = false,
    actions,
    filterRef = $bindable(),
  }: {
    title: string;
    count?: number;
    subtitle?: string;
    segments?: Segment[];
    activeSegment?: string;
    onSegment?: (id: string) => void;
    filter?: string;
    filterPlaceholder?: string;
    /** Keys the session-persisted filter value. */
    viewId?: string;
    /** Shows the 2px indeterminate bar while a refresh runs with rows on screen. */
    loading?: boolean;
    actions?: Snippet;
    filterRef?: HTMLInputElement;
  } = $props();

  const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

  onMount(() => {
    if (viewId && !filter) filter = rememberedFilter(viewId);
  });

  $effect(() => {
    if (viewId) sessionFilters.set(viewId, filter);
  });

  function onFilterKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      if (filter) filter = '';
      else filterRef?.blur();
    }
  }

  function handleWindowKeydown(e: KeyboardEvent) {
    // ⌘F / Ctrl+F focuses the filter, even from another input.
    if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      filterRef?.focus();
      filterRef?.select();
      return;
    }
    // 1-5 select segments when not typing.
    if (!segments?.length || e.metaKey || e.ctrlKey || e.altKey || isTypingTarget(document.activeElement)) return;
    const n = Number(e.key);
    if (Number.isInteger(n) && n >= 1 && n <= Math.min(5, segments.length)) {
      const seg = segments[n - 1];
      if (seg.count > 0 || n === 1) {
        e.preventDefault();
        onSegment?.(seg.id);
      }
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<header
  class="relative shrink-0 flex flex-wrap items-center gap-x-(--toolbar-gap) gap-y-2 bg-app border-b border-line px-(--page-px) py-(--page-py)"
>
  <div class="flex items-baseline gap-2 min-w-0 mr-auto">
    <h1 class="font-semibold text-primary tracking-[-0.025em] truncate" style="font-size: var(--title-font); line-height: 1.3">
      {title}
    </h1>
    {#if count !== undefined}<span class="text-sm text-muted tabular-nums">{count}</span>{/if}
    {#if subtitle}<span class="text-sm text-muted truncate">{subtitle}</span>{/if}
  </div>

  {#if segments?.length}
    <div class="inline-flex items-center gap-0.5 p-0.5 max-w-full overflow-x-auto rounded-md border border-line-strong bg-app" role="radiogroup" aria-label="Status filter">
      {#each segments as seg, i (seg.id)}
        {@const active = seg.id === activeSegment}
        {@const disabled = seg.count === 0 && i !== 0}
        <button
          type="button"
          role="radio"
          aria-checked={active}
          {disabled}
          title={i < 5 ? `${seg.label} · ${i + 1}` : seg.label}
          class="seg-btn inline-flex items-center gap-1.5 px-2 rounded-sm text-sm transition-colors duration-100
            {active
            ? 'bg-active text-primary'
            : disabled
              ? 'text-faint cursor-default'
              : 'text-secondary hover:text-primary'}"
          onclick={() => onSegment?.(seg.id)}
        >
          {#if seg.health}<StatusIcon health={seg.health} colored={!disabled} />{/if}
          <span>{seg.label}</span>
          <span class="tabular-nums {active ? 'text-secondary' : 'text-muted'}">{seg.count}</span>
        </button>
      {/each}
    </div>
  {/if}

  <div class="flex items-center gap-2 w-full sm:w-auto">
    <div class="relative w-full sm:w-[240px]">
      <svg class="absolute left-2 top-1/2 -translate-y-1/2 text-muted pointer-events-none" width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true">
        <circle cx="7" cy="7" r="4.75" /><path d="M13.5 13.5l-3-3" />
      </svg>
      <input
        bind:this={filterRef}
        bind:value={filter}
        type="text"
        placeholder={filterPlaceholder}
        aria-label={filterPlaceholder}
        spellcheck="false"
        autocomplete="off"
        onkeydown={onFilterKeydown}
        class="filter-input w-full h-(--control-h) pl-7 pr-8 rounded-md bg-app border border-line-strong hover:border-faint text-sm text-primary transition-colors duration-100"
      />
      <div class="absolute right-1.5 top-1/2 -translate-y-1/2 flex items-center">
        {#if filter}
          <button
            type="button"
            class="w-5 h-5 inline-flex items-center justify-center rounded-sm text-muted hover:text-primary hover:bg-hover"
            aria-label="Clear filter"
            onclick={() => {
              filter = '';
              filterRef?.focus();
            }}
          >
            <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.25" stroke-linecap="round" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" /></svg>
          </button>
        {:else}
          <Kbd keys={isMac ? '⌘F' : 'Ctrl F'} />
        {/if}
      </div>
    </div>
    {#if actions}{@render actions()}{/if}
  </div>

  {#if loading}
    <div class="absolute left-0 right-0 -bottom-px h-0.5 overflow-hidden" aria-hidden="true">
      <div class="progress h-full w-1/3 bg-accent"></div>
    </div>
  {/if}
</header>

<style>
  .seg-btn {
    height: calc(var(--control-h) - 6px);
  }
  .filter-input:focus-visible {
    outline: none;
    border-color: var(--ui-accent);
    box-shadow: 0 0 0 1px var(--ui-accent);
  }
  .progress {
    animation: indeterminate 1.2s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  @keyframes indeterminate {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(300%);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .progress {
      animation: none;
      width: 100%;
    }
  }
</style>
