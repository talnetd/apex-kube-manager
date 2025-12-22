<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    contexts,
    currentContext,
    namespaces,
    selectedNamespace,
    pulseMetrics,
    isLoading,
    loadContexts,
    loadNamespaces,
    switchContext,
    loadPulseMetrics,
    triggerRefresh,
    startContextPolling,
    stopContextPolling,
  } from '../stores/kubernetes';
  import { searchQuery, globalSearchOpen } from '../stores/search';
  import { activePortForwardCount, portForwardPanelOpen } from '../stores/portforward';
  import PortForwardPanel from './PortForwardPanel.svelte';
  import WindowControls from './ui/WindowControls.svelte';
  import ThemeToggle from './ui/ThemeToggle.svelte';

  let showContextDropdown = $state(false);
  let showNamespaceDropdown = $state(false);

  function openGlobalSearch() {
    globalSearchOpen.set(true);
  }

  onMount(async () => {
    await loadContexts();
    await loadNamespaces();
    await loadPulseMetrics();
    startContextPolling();
  });

  onDestroy(() => {
    stopContextPolling();
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
</script>

<header class="h-14 bg-bg-secondary border-b border-border-subtle flex items-center px-4 gap-3">
  <!-- Context/Cluster Selector -->
  <div class="relative">
    <button
      onclick={() => { showContextDropdown = !showContextDropdown; showNamespaceDropdown = false; }}
      class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-bg-tertiary border border-border-subtle hover:border-accent-primary transition-colors"
    >
      <div class="w-2 h-2 rounded-full bg-accent-success"></div>
      <span class="text-sm text-text-primary max-w-[180px] truncate">{$pulseMetrics?.cluster || $currentContext || 'Select Context'}</span>
      <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    {#if showContextDropdown}
      <div class="absolute top-full left-0 mt-1 w-72 bg-bg-card border border-border-subtle rounded-lg shadow-xl z-50">
        <div class="p-2 border-b border-border-subtle">
          <span class="text-xs text-text-muted uppercase tracking-wide">Kubernetes Contexts</span>
        </div>
        <ul class="max-h-60 overflow-y-auto py-1">
          {#each $contexts as ctx}
            <li>
              <button
                onclick={() => handleContextChange(ctx.name)}
                class="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-bg-tertiary transition-colors {ctx.is_current ? 'text-accent-primary' : 'text-text-primary'}"
              >
                {#if ctx.is_current}
                  <svg class="w-4 h-4 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                {:else}
                  <div class="w-4"></div>
                {/if}
                <div class="flex-1 min-w-0">
                  <div class="text-sm truncate">{ctx.name}</div>
                  <div class="text-xs text-text-muted truncate">{ctx.cluster}</div>
                </div>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>

  <!-- Namespace Selector -->
  <div class="relative">
    <button
      onclick={() => { showNamespaceDropdown = !showNamespaceDropdown; showContextDropdown = false; }}
      class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-bg-tertiary border border-border-subtle hover:border-accent-primary transition-colors"
    >
      <span class="text-sm text-text-primary">{$selectedNamespace || '*'}</span>
      <svg class="w-4 h-4 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    {#if showNamespaceDropdown}
      <div class="absolute top-full left-0 mt-1 w-48 bg-bg-card border border-border-subtle rounded-lg shadow-xl z-50">
        <ul class="max-h-60 overflow-y-auto py-1">
          <li>
            <button
              onclick={() => handleNamespaceChange(null)}
              class="w-full px-3 py-2 text-left text-sm hover:bg-bg-tertiary transition-colors {!$selectedNamespace ? 'text-accent-primary' : 'text-text-primary'}"
            >
              *
            </button>
          </li>
          {#each $namespaces as ns}
            <li>
              <button
                onclick={() => handleNamespaceChange(ns)}
                class="w-full px-3 py-2 text-left text-sm hover:bg-bg-tertiary transition-colors {$selectedNamespace === ns ? 'text-accent-primary' : 'text-text-primary'}"
              >
                {ns}
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}
  </div>

  <!-- Divider -->
  <div class="w-px h-6 bg-border-subtle"></div>

  <!-- Cluster Info Pills -->
  {#if $pulseMetrics}
    <!-- K8s Version -->
    <div class="flex items-center gap-1.5 px-2.5 py-1 bg-bg-tertiary rounded-md border border-accent-success/30">
      <span class="text-accent-success font-mono text-xs">{$pulseMetrics.k8s_version}</span>
    </div>

    <!-- Nodes -->
    <div class="flex items-center gap-1.5 px-2.5 py-1 bg-bg-tertiary rounded-md border border-border-subtle">
      <svg class="w-3.5 h-3.5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
      </svg>
      <span class="text-xs text-text-muted">Nodes</span>
      <span class="text-xs text-text-primary font-medium">{$pulseMetrics.nodes}</span>
    </div>

    <!-- Namespaces -->
    <div class="flex items-center gap-1.5 px-2.5 py-1 bg-bg-tertiary rounded-md border border-border-subtle">
      <svg class="w-3.5 h-3.5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
      </svg>
      <span class="text-xs text-text-muted">NS</span>
      <span class="text-xs text-text-primary font-medium">{$pulseMetrics.namespaces}</span>
    </div>

  {/if}

  <!-- Spacer / Drag Region -->
  <div class="flex-1 h-full cursor-grab active:cursor-grabbing" data-tauri-drag-region></div>

  <!-- Search -->
  <button
    type="button"
    onclick={openGlobalSearch}
    class="p-2 rounded-lg hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
    title="Search (⌘K)"
  >
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
  </button>

  <!-- Refresh Button -->
  <button
    onclick={() => { triggerRefresh(); loadPulseMetrics($selectedNamespace); }}
    class="p-2 rounded-lg hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
    title="Refresh"
  >
    <svg class="w-5 h-5 {$isLoading ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
    </svg>
  </button>

  <!-- Port Forward Button -->
  <div class="relative">
    <button
      onclick={() => portForwardPanelOpen.update(v => !v)}
      class="relative p-2 rounded-lg hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors {$portForwardPanelOpen ? 'bg-bg-tertiary text-accent-primary' : ''}"
      title="Port Forwards (⌘P)"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
      </svg>
      {#if $activePortForwardCount > 0}
        <span class="absolute -top-1 -right-1 w-4 h-4 bg-accent-primary text-white text-xs rounded-full flex items-center justify-center">
          {$activePortForwardCount}
        </span>
      {/if}
    </button>
    <PortForwardPanel
      isOpen={$portForwardPanelOpen}
      onClose={() => portForwardPanelOpen.set(false)}
    />
  </div>

  <!-- Theme Toggle -->
  <ThemeToggle />

  <!-- Window Controls -->
  <div class="ml-2 pl-2 border-l border-border-subtle">
    <WindowControls />
  </div>
</header>
