<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import {
    searchQuery,
    globalSearchOpen,
    globalSearchResults,
    countResults,
    resourceDisplayNames,
    resourceTypeMap,
    type SearchResults,
  } from '../stores/search';
  import { currentContext, selectedNamespace, loadAllResources } from '../stores/kubernetes';

  let searchInput: HTMLInputElement;
  let selectedIndex = $state(0);
  let listContainer: HTMLDivElement;
  let isLoadingResources = $state(false);

  // Flatten all results into a single list for command palette style
  const flatResults = $derived(() => {
    const results = $globalSearchResults;
    if (!results) return [];

    const flat: {
      category: keyof SearchResults;
      item: { name: string; namespace?: string; status?: string };
    }[] = [];

    // Order: pods first, then deployments, services, etc.
    const order: (keyof SearchResults)[] = [
      'pods', 'deployments', 'statefulsets', 'daemonsets', 'replicasets',
      'jobs', 'cronjobs', 'services', 'ingresses', 'configmaps', 'secrets',
      'networkPolicies', 'hpas', 'pvs', 'pvcs', 'namespaces', 'nodes', 'serviceAccounts'
    ];

    for (const category of order) {
      const items = results[category];
      if (items && items.length > 0) {
        for (const item of items) {
          flat.push({
            category,
            item: item as { name: string; namespace?: string; status?: string }
          });
        }
      }
    }

    return flat;
  });

  function close() {
    globalSearchOpen.set(false);
    searchQuery.set('');
    selectedIndex = 0;
  }

  async function openResource(category: keyof SearchResults, item: { name: string; namespace?: string }) {
    const resourceType = resourceTypeMap[category];
    const isClusterScoped = ['pv', 'namespace', 'node'].includes(resourceType);

    try {
      await invoke('open_resource_detail', {
        resourceType,
        name: item.name,
        namespace: isClusterScoped ? '' : (item.namespace || ''),
        context: $currentContext,
      });
      close();
    } catch (e) {
      console.error('Failed to open resource detail:', e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    const flat = flatResults();
    const total = flat.length;

    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        close();
        break;
      case 'ArrowDown':
        e.preventDefault();
        if (total > 0) {
          selectedIndex = (selectedIndex + 1) % total;
          scrollToSelected();
        }
        break;
      case 'ArrowUp':
        e.preventDefault();
        if (total > 0) {
          selectedIndex = (selectedIndex - 1 + total) % total;
          scrollToSelected();
        }
        break;
      case 'Enter':
        e.preventDefault();
        if (flat[selectedIndex]) {
          const { category, item } = flat[selectedIndex];
          openResource(category, item);
        }
        break;
    }
  }

  function scrollToSelected() {
    if (listContainer) {
      const selected = listContainer.querySelector(`[data-index="${selectedIndex}"]`);
      if (selected) {
        selected.scrollIntoView({ block: 'nearest' });
      }
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }

  onMount(() => {
    searchInput?.focus();
  });

  // Focus input and load resources when modal opens
  $effect(() => {
    if ($globalSearchOpen) {
      searchInput?.focus();
      // Load all resources so search has data
      isLoadingResources = true;
      loadAllResources($selectedNamespace).finally(() => {
        isLoadingResources = false;
      });
    }
  });

  // Reset selection when results change
  $effect(() => {
    const _ = $globalSearchResults;
    selectedIndex = 0;
  });

  // Get status color for resource
  function getStatusColor(item: { status?: string }): string {
    if (item.status) {
      const status = item.status.toLowerCase();
      if (status === 'running' || status === 'active' || status === 'bound' || status === 'ready' || status === 'available' || status === 'complete' || status === 'succeeded') {
        return 'bg-accent-success';
      }
      if (status === 'pending' || status === 'terminating' || status === 'released') {
        return 'bg-accent-warning';
      }
      if (status === 'failed' || status === 'error' || status === 'lost' || status === 'notready' || status === 'crashloopbackoff') {
        return 'bg-accent-error';
      }
    }
    return 'bg-accent-primary';
  }

  // Get badge color for resource type
  function getBadgeColor(category: keyof SearchResults): string {
    switch (category) {
      case 'pods':
        return 'bg-blue-500/20 text-blue-400';
      case 'deployments':
      case 'statefulsets':
      case 'daemonsets':
      case 'replicasets':
        return 'bg-purple-500/20 text-purple-400';
      case 'jobs':
      case 'cronjobs':
        return 'bg-amber-500/20 text-amber-400';
      case 'services':
      case 'ingresses':
        return 'bg-cyan-500/20 text-cyan-400';
      case 'configmaps':
      case 'secrets':
        return 'bg-emerald-500/20 text-emerald-400';
      case 'networkPolicies':
        return 'bg-rose-500/20 text-rose-400';
      case 'hpas':
        return 'bg-orange-500/20 text-orange-400';
      case 'pvs':
      case 'pvcs':
        return 'bg-indigo-500/20 text-indigo-400';
      case 'namespaces':
      case 'nodes':
      case 'serviceAccounts':
        return 'bg-slate-500/20 text-slate-400';
      default:
        return 'bg-gray-500/20 text-gray-400';
    }
  }

  // Short display name for badge
  function getShortName(category: keyof SearchResults): string {
    const names: Record<keyof SearchResults, string> = {
      pods: 'Pod',
      deployments: 'Deploy',
      statefulsets: 'STS',
      daemonsets: 'DS',
      replicasets: 'RS',
      jobs: 'Job',
      cronjobs: 'CronJob',
      services: 'Service',
      ingresses: 'Ingress',
      networkPolicies: 'NetPol',
      configmaps: 'CM',
      secrets: 'Secret',
      hpas: 'HPA',
      pvs: 'PV',
      pvcs: 'PVC',
      namespaces: 'NS',
      nodes: 'Node',
      serviceAccounts: 'SA',
    };
    return names[category];
  }
</script>

{#if $globalSearchOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-start justify-center pt-[15vh] bg-black/60 backdrop-blur-sm"
    onclick={handleBackdropClick}
  >
    <div
      class="w-full max-w-xl bg-bg-primary border border-border-subtle rounded-xl shadow-2xl overflow-hidden"
      onkeydown={handleKeydown}
    >
      <!-- Search Input -->
      <div class="p-3 border-b border-border-subtle">
        <div class="relative">
          <svg
            class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
          <input
            bind:this={searchInput}
            type="text"
            placeholder="Search resources..."
            class="w-full pl-9 pr-16 py-2 bg-transparent text-text-primary placeholder-text-muted focus:outline-none text-sm"
            bind:value={$searchQuery}
          />
          <div class="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-2">
            {#if $searchQuery}
              <span class="text-xs text-text-muted">
                {countResults($globalSearchResults)}
              </span>
            {/if}
            <kbd class="px-1.5 py-0.5 text-[10px] text-text-muted bg-bg-tertiary rounded border border-border-subtle">
              esc
            </kbd>
          </div>
        </div>
      </div>

      <!-- Results -->
      <div bind:this={listContainer} class="max-h-80 overflow-y-auto">
        {#if !$searchQuery}
          <div class="px-3 py-8 text-center text-text-muted">
            {#if isLoadingResources}
              <div class="flex items-center justify-center gap-2">
                <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                </svg>
                <span class="text-sm">Loading resources...</span>
              </div>
            {:else}
              <p class="text-sm">Type to search pods, deployments, services...</p>
            {/if}
          </div>
        {:else if flatResults().length === 0}
          <div class="px-3 py-8 text-center text-text-muted">
            <p class="text-sm">No results for "{$searchQuery}"</p>
          </div>
        {:else}
          <div class="py-1">
            {#each flatResults() as { category, item }, idx}
              <button
                type="button"
                data-index={idx}
                class="w-full px-3 py-2 flex items-center gap-3 hover:bg-bg-secondary transition-colors text-left {idx === selectedIndex ? 'bg-bg-secondary' : ''}"
                onclick={() => openResource(category, item)}
              >
                <!-- Status dot -->
                <div class="w-1.5 h-1.5 rounded-full flex-shrink-0 {getStatusColor(item)}"></div>

                <!-- Name -->
                <span class="flex-1 text-sm text-text-primary truncate">{item.name}</span>

                <!-- Namespace (if applicable) -->
                {#if item.namespace}
                  <span class="text-xs text-text-muted truncate max-w-24">{item.namespace}</span>
                {/if}

                <!-- Resource type badge -->
                <span class="px-1.5 py-0.5 text-[10px] font-medium rounded {getBadgeColor(category)}">
                  {getShortName(category)}
                </span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-3 py-2 border-t border-border-subtle bg-bg-secondary/50">
        <div class="flex items-center justify-between text-[10px] text-text-muted">
          <div class="flex items-center gap-3">
            <span class="flex items-center gap-1">
              <kbd class="px-1 py-0.5 bg-bg-tertiary rounded border border-border-subtle">↑↓</kbd>
              <span>navigate</span>
            </span>
            <span class="flex items-center gap-1">
              <kbd class="px-1 py-0.5 bg-bg-tertiary rounded border border-border-subtle">↵</kbd>
              <span>open</span>
            </span>
          </div>
          <span class="flex items-center gap-1">
            <kbd class="px-1 py-0.5 bg-bg-tertiary rounded border border-border-subtle">⌘K</kbd>
            <span>search</span>
          </span>
        </div>
      </div>
    </div>
  </div>
{/if}
