<script lang="ts">
  // Header (spec 4.3, 13.2, 13.3; approved Geist preview): outlined context
  // and namespace controls, PROD tag, linked-windows marker, ⌘K search box,
  // then icon buttons. sm: menu button, namespace folds into the context control.
  import { onMount, onDestroy } from 'svelte';
  import {
    contexts,
    currentContext,
    namespaces,
    selectedNamespace,
    isSwitchingContext,
    loadContexts,
    loadNamespaces,
    switchContext,
    loadPulseMetrics,
    triggerRefresh,
    startContextPolling,
    stopContextPolling,
  } from '../stores/kubernetes';
  import { globalSearchOpen } from '../stores/search';
  import { activePortForwardCount, portForwardPanelOpen } from '../stores/portforward';
  import { pulseRailOpen } from '../stores/rail';
  import { navOpen } from '../stores/navigation';
  import { updateInfo, initUpdateCheck } from '../stores/updates';
  import { currentContextIsProd, prodOverrides, resolveProd } from '../stores/production';
  import { mainWindowCount } from '../stores/windows';
  import { toggleDensity, densityLabel } from '../stores/density';
  import { toast } from '../stores/toast';
  import { open } from '@tauri-apps/plugin-shell';
  import PortForwardPanel from './PortForwardPanel.svelte';
  import WindowControls from './ui/WindowControls.svelte';
  import ThemeToggle from './ui/ThemeToggle.svelte';
  import NotificationToggle from './ui/NotificationToggle.svelte';
  import Icon from './shell/Icon.svelte';

  let showContextDropdown = $state(false);
  let showNamespaceDropdown = $state(false);
  let showUpdateDropdown = $state(false);

  const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);

  function closeDropdowns() {
    showContextDropdown = false;
    showNamespaceDropdown = false;
    showUpdateDropdown = false;
  }

  async function openReleasePage() {
    if ($updateInfo?.release_url) {
      await open($updateInfo.release_url);
      showUpdateDropdown = false;
    }
  }

  onMount(async () => {
    await loadContexts();
    await loadNamespaces();
    await loadPulseMetrics();
    startContextPolling();
    initUpdateCheck();
    window.addEventListener('click', closeDropdowns);
  });

  onDestroy(() => {
    stopContextPolling();
    window.removeEventListener('click', closeDropdowns);
  });

  async function handleContextChange(contextName: string) {
    showContextDropdown = false;
    await switchContext(contextName);
    // Load pulse metrics for all namespaces (namespace was reset in switchContext)
    await loadPulseMetrics(null);
  }

  function handleNamespaceChange(ns: string | null) {
    selectedNamespace.set(ns);
    showNamespaceDropdown = false;
  }

  function density() {
    const next = toggleDensity();
    toast(`Density: ${densityLabel(next)}`, { duration: 1500 });
  }

  const wins = $derived($mainWindowCount);
  const ctxLabel = $derived($isSwitchingContext ? 'Switching…' : $currentContext || 'Select context');
</script>

<header class="top" class:switching={$isSwitchingContext}>
  <button
    type="button"
    class="ib menu-btn"
    aria-label="Open navigation"
    onclick={(e) => {
      e.stopPropagation();
      navOpen.update((v) => !v);
    }}><Icon name="menu" /></button
  >

  <!-- Context -->
  <div class="relative min-w-0">
    <button
      type="button"
      class="ctl"
      class:prod={$currentContextIsProd}
      aria-haspopup="menu"
      aria-expanded={showContextDropdown}
      aria-label={`Context ${$currentContext}${$currentContextIsProd ? ', production' : ''}${wins > 1 ? `, shared by ${wins} windows` : ''}. Change context`}
      disabled={$isSwitchingContext}
      onclick={(e) => {
        e.stopPropagation();
        showContextDropdown = !showContextDropdown;
        showNamespaceDropdown = false;
      }}
    >
      {#if $currentContextIsProd}<Icon name="shield" size={14} class="text-warning" />{/if}
      <span class="nm">{ctxLabel}</span>
      <span class="nsin">/ {$selectedNamespace || '*'}</span>
      <span class="dot" aria-hidden="true"></span>
      {#if $currentContextIsProd}<span class="prodtag">PROD</span>{/if}
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" class="text-muted shrink-0" aria-hidden="true"><path d="M2.5 3.5L5 6l2.5-2.5" /></svg>
      {#if wins > 1}
        <span class="lnk" title={`Shared by all ${wins} Apex windows`}>
          <Icon name="link" size={12} />{wins}<span class="lw"> windows</span>
        </span>
      {/if}
    </button>

    {#if showContextDropdown}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="menu absolute top-full left-0 mt-1 w-72" onclick={(e) => e.stopPropagation()}>
        <div class="lblm">{wins > 1 ? `Applies to all windows (${wins})` : 'Contexts'}</div>
        <ul class="max-h-72 overflow-y-auto">
          {#each $contexts as ctx (ctx.name)}
            <li>
              <button type="button" class="mi" onclick={() => handleContextChange(ctx.name)}>
                <span class="ck">
                  {#if ctx.is_current}<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>{/if}
                </span>
                <span class="flex-1 min-w-0 text-left">
                  <span class="block truncate">{ctx.name}</span>
                  <span class="block truncate text-xs text-muted">{ctx.cluster}</span>
                </span>
                {#if resolveProd(ctx.name, $prodOverrides)}<span class="prodtag">PROD</span>{/if}
              </button>
            </li>
          {/each}
        </ul>
        <!-- sm: the namespace control is folded into this menu -->
        <div class="sm-only">
          <div class="sepl"></div>
          <div class="lblm">Namespace</div>
          <ul class="max-h-56 overflow-y-auto">
            <li>
              <button type="button" class="mi" onclick={() => { handleNamespaceChange(null); showContextDropdown = false; }}>
                <span class="ck">{#if !$selectedNamespace}<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>{/if}</span>
                <span>All namespaces</span>
              </button>
            </li>
            {#each $namespaces as ns (ns)}
              <li>
                <button type="button" class="mi" onclick={() => { handleNamespaceChange(ns); showContextDropdown = false; }}>
                  <span class="ck">{#if $selectedNamespace === ns}<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>{/if}</span>
                  <span class="truncate">{ns}</span>
                </button>
              </li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  </div>

  <!-- Namespace -->
  <div class="relative min-w-0 ns-wrap">
    <button
      type="button"
      class="ctl"
      aria-haspopup="menu"
      aria-expanded={showNamespaceDropdown}
      onclick={(e) => {
        e.stopPropagation();
        showNamespaceDropdown = !showNamespaceDropdown;
        showContextDropdown = false;
      }}
    >
      <span class="text-muted">ns:</span>
      <span class="nm">{$selectedNamespace || 'All namespaces'}</span>
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" class="text-muted shrink-0" aria-hidden="true"><path d="M2.5 3.5L5 6l2.5-2.5" /></svg>
    </button>

    {#if showNamespaceDropdown}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="menu absolute top-full left-0 mt-1 w-56" onclick={(e) => e.stopPropagation()}>
        <ul class="max-h-72 overflow-y-auto">
          <li>
            <button type="button" class="mi" onclick={() => handleNamespaceChange(null)}>
              <span class="ck">{#if !$selectedNamespace}<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>{/if}</span>
              <span>All namespaces</span>
            </button>
          </li>
          {#each $namespaces as ns (ns)}
            <li>
              <button type="button" class="mi" onclick={() => handleNamespaceChange(ns)}>
                <span class="ck">{#if $selectedNamespace === ns}<svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3.5 8.5l3 3 6-7" /></svg>{/if}</span>
                <span class="truncate">{ns}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>

  <!-- Drag region between the controls and the search box -->
  <div class="drag flex-1 self-stretch" data-tauri-drag-region></div>

  <button type="button" class="ctl search" onclick={() => globalSearchOpen.set(true)} aria-label="Search or jump to">
    <span class="flex items-center gap-1.5 min-w-0"><Icon name="search" size={14} /><span class="st truncate">Search or jump to…</span></span>
    <span class="kbd">{isMac ? '⌘K' : 'Ctrl K'}</span>
  </button>

  <button
    type="button"
    class="ib hide-s"
    aria-label="Refresh"
    title="Refresh · r"
    onclick={() => {
      triggerRefresh();
      loadPulseMetrics($selectedNamespace);
    }}><Icon name="restart" /></button
  >
  <button type="button" class="ib hide-s" aria-label="Toggle density" title="Density · ⌘⇧D" onclick={density}><Icon name="density" /></button>
  <button
    type="button"
    class="ib hide-s"
    aria-label="Pulse rail: capacity and events"
    aria-pressed={$pulseRailOpen}
    title="Pulse rail · ⌘E"
    onclick={() => pulseRailOpen.update((v) => !v)}><Icon name="rail" /></button
  >

  <div class="relative">
    <button
      type="button"
      class="ib relative hide-s"
      aria-label="Port forwards"
      aria-pressed={$portForwardPanelOpen}
      title="Port forwards · ⌘P"
      onclick={() => portForwardPanelOpen.update((v) => !v)}
    >
      <Icon name="forward" />
      {#if $activePortForwardCount > 0}
        <span class="absolute -top-0.5 -right-0.5 font-mono text-[10px] leading-none text-secondary">{$activePortForwardCount}</span>
      {/if}
    </button>
    <PortForwardPanel isOpen={$portForwardPanelOpen} onClose={() => portForwardPanelOpen.set(false)} />
  </div>

  {#if $updateInfo?.update_available}
    <div class="relative">
      <button
        type="button"
        class="ib relative"
        aria-label="Update available"
        title="Update available"
        onclick={(e) => {
          e.stopPropagation();
          showUpdateDropdown = !showUpdateDropdown;
        }}
      >
        <Icon name="download" />
        <span class="absolute top-1 right-1 w-1.5 h-1.5 rounded-full bg-accent" aria-hidden="true"></span>
      </button>

      {#if showUpdateDropdown}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
        <div class="menu absolute top-full right-0 mt-1 w-64 p-3!" onclick={(e) => e.stopPropagation()}>
          <div class="text-sm font-medium text-primary mb-2">Update available</div>
          <div class="flex justify-between text-sm mb-1">
            <span class="text-muted">Current</span><span class="font-mono text-secondary">v{$updateInfo.current_version}{$updateInfo.build_label ? ` (${$updateInfo.build_label})` : ''}</span>
          </div>
          <div class="flex justify-between text-sm mb-3">
            <span class="text-muted">Latest</span><span class="font-mono text-primary">v{$updateInfo.latest_version}</span>
          </div>
          <button
            type="button"
            onclick={openReleasePage}
            class="w-full h-7 inline-flex items-center justify-center gap-1.5 rounded-md bg-btn text-on-btn text-sm font-medium hover:opacity-90"
          >
            <Icon name="open" size={14} />View release
          </button>
        </div>
      {/if}
    </div>
  {/if}

  <span class="hide-s"><NotificationToggle /></span>
  <span class="hide-s"><ThemeToggle /></span>

  <div class="ml-1 pl-2 border-l border-line self-center">
    <WindowControls />
  </div>

  <!-- Context switch progress (the one allowed motion, spec 13.1) -->
  {#if $isSwitchingContext}
    <div class="absolute bottom-0 left-0 right-0 h-0.5 overflow-hidden" aria-hidden="true">
      <div class="h-full bg-accent context-switch-bar"></div>
    </div>
  {/if}
</header>

<style>
  .top {
    grid-column: 2;
    grid-row: 1;
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    min-width: 0;
    background: var(--ui-app);
    border-bottom: 1px solid var(--ui-line);
  }
  .ctl {
    height: 28px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px;
    min-width: 0;
    max-width: 100%;
    white-space: nowrap;
    font-size: 13px;
    color: var(--ui-primary);
    background: var(--ui-app);
    border: 1px solid var(--ui-line-strong);
    border-radius: 6px;
  }
  .ctl:hover:not(:disabled) {
    background: var(--ui-hover);
    border-color: var(--ui-faint);
  }
  .ctl:disabled {
    cursor: progress;
  }
  .ctl.prod {
    border-color: color-mix(in srgb, var(--ui-warning-solid) 45%, transparent);
  }
  .ctl .nm {
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 180px;
  }
  .dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
    background: var(--ui-faint);
  }
  .nsin {
    display: none;
    color: var(--ui-muted);
  }
  .lnk {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 11px;
    color: var(--ui-muted);
    padding-left: 6px;
    margin-left: 2px;
    border-left: 1px solid var(--ui-line-strong);
    font-variant-numeric: tabular-nums;
  }
  .search {
    flex: 0 1 280px;
    justify-content: space-between;
    color: var(--ui-muted);
  }
  .kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 14px;
    padding: 0 4px;
    border: 1px solid var(--ui-line-strong);
    border-radius: 4px;
    color: var(--ui-muted);
  }
  .ib {
    width: 28px;
    height: 28px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 6px;
    color: var(--ui-secondary);
  }
  .ib:hover {
    background: var(--ui-hover);
    color: var(--ui-primary);
  }
  .ib[aria-pressed='true'] {
    background: var(--ui-active);
    color: var(--ui-primary);
  }
  .menu-btn {
    display: none;
  }
  .drag {
    min-width: 24px;
  }
  .sm-only {
    display: none;
  }
  .sepl {
    height: 1px;
    margin: 4px 0;
    background: var(--ui-divider);
  }
  .menu {
    z-index: 85;
    padding: 4px;
    background: var(--ui-surface);
    border: 1px solid var(--ui-line-strong);
    border-radius: 6px;
  }
  .lblm {
    padding: 6px 8px 2px;
    font-size: 11px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--ui-muted);
  }
  .mi {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 30px;
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 13px;
    color: var(--ui-primary);
    text-align: left;
  }
  .mi:hover {
    background: var(--ui-active);
  }
  .ck {
    width: 14px;
    flex: none;
    color: var(--ui-accent-text);
  }
  .context-switch-bar {
    width: 40%;
    animation: context-switch-progress 1.2s ease-in-out infinite;
  }
  @keyframes context-switch-progress {
    0% {
      transform: translateX(-150%);
    }
    100% {
      transform: translateX(350%);
    }
  }

  @media (max-width: 639.98px) {
    .top {
      grid-column: 1;
    }
    .menu-btn {
      display: grid;
    }
    .ns-wrap,
    .hide-s {
      display: none !important;
    }
    .nsin {
      display: inline;
    }
    .sm-only {
      display: block;
    }
    .dot {
      display: none;
    }
    .ctl .nm {
      max-width: 78px;
    }
    .lw {
      display: none;
    }
    .search {
      flex: 0 0 auto;
    }
    .search .st,
    .search .kbd {
      display: none;
    }
  }
</style>
