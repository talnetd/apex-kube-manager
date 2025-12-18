<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    portForwards,
    activePortForwardCount,
    loadPortForwards,
    startPortForward,
    stopPortForward,
    getResourcePorts,
    startPolling,
    stopPolling,
    type PortForwardInfo,
    type ResourceType,
    type AvailablePort
  } from '../stores/portforward';
  import {
    currentContext,
    namespaces,
    selectedNamespace,
    pods,
    services,
    loadPods,
    loadServices
  } from '../stores/kubernetes';
  import CustomSelect from './ui/CustomSelect.svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  // Form state
  let showNewForm = $state(false);
  let resourceType = $state<ResourceType>('pod');
  let selectedNamespaceForm = $state<string>('');
  let selectedResource = $state<string>('');
  let localPort = $state<number>(8080);
  let remotePort = $state<number>(0);
  let isStarting = $state(false);
  let formError = $state<string | null>(null);
  let isLoadingResources = $state(false);
  let availablePorts = $state<AvailablePort[]>([]);
  let isLoadingPorts = $state(false);

  onMount(() => {
    loadPortForwards();
    startPolling(3000);
  });

  onDestroy(() => {
    stopPolling();
  });

  // Load resources when namespace or type changes
  async function loadResourcesForNamespace() {
    if (!selectedNamespaceForm) return;

    isLoadingResources = true;
    selectedResource = '';
    try {
      if (resourceType === 'pod') {
        await loadPods(selectedNamespaceForm);
      } else {
        await loadServices(selectedNamespaceForm);
      }
    } catch (e) {
      console.error('Failed to load resources:', e);
    } finally {
      isLoadingResources = false;
    }
  }

  // Get available resources based on type
  function getAvailableResources(): string[] {
    if (resourceType === 'pod') {
      return $pods
        .filter(p => p.namespace === selectedNamespaceForm && p.status === 'Running')
        .map(p => p.name);
    } else {
      return $services
        .filter(s => s.namespace === selectedNamespaceForm)
        .map(s => s.name);
    }
  }

  // Load available ports when resource is selected
  async function loadAvailablePorts() {
    if (!selectedResource || !selectedNamespaceForm || !$currentContext) {
      availablePorts = [];
      return;
    }

    isLoadingPorts = true;
    try {
      availablePorts = await getResourcePorts(
        $currentContext,
        selectedNamespaceForm,
        resourceType,
        selectedResource
      );
      // Auto-select first port if available
      if (availablePorts.length > 0) {
        remotePort = availablePorts[0].port;
        localPort = availablePorts[0].port;
      }
    } catch (e) {
      console.error('Failed to load ports:', e);
      availablePorts = [];
    } finally {
      isLoadingPorts = false;
    }
  }

  // Handle resource selection change
  function handleResourceChange() {
    loadAvailablePorts();
  }

  async function handleStartForward() {
    if (!selectedResource || !selectedNamespaceForm || !$currentContext) {
      formError = 'Please select a resource';
      return;
    }

    if (!remotePort || remotePort < 1 || remotePort > 65535) {
      formError = 'Please select a remote port';
      return;
    }

    if (localPort < 1 || localPort > 65535) {
      formError = 'Local port must be 1-65535';
      return;
    }

    isStarting = true;
    formError = null;

    try {
      await startPortForward(
        $currentContext,
        selectedNamespaceForm,
        resourceType,
        selectedResource,
        localPort,
        remotePort
      );
      resetForm();
    } catch (e) {
      formError = String(e).replace('Custom { message: "', '').replace('" }', '');
    } finally {
      isStarting = false;
    }
  }

  function resetForm() {
    showNewForm = false;
    selectedResource = '';
    localPort = 8080;
    remotePort = 0;
    formError = null;
    availablePorts = [];
  }

  async function handleStopForward(id: string) {
    try {
      await stopPortForward(id);
    } catch (e) {
      console.error('Failed to stop forward:', e);
    }
  }

  function openNewForm() {
    showNewForm = true;
    selectedNamespaceForm = $selectedNamespace || '';
    if (selectedNamespaceForm) {
      loadResourcesForNamespace();
    }
  }

  function getStatusIndicator(status: string): string {
    switch (status) {
      case 'active': return 'bg-accent-success';
      case 'starting': return 'bg-accent-warning animate-pulse';
      case 'error': return 'bg-accent-error';
      default: return 'bg-text-muted';
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const k = 1024;
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-40"
    onclick={handleBackdropClick}
    onkeydown={(e) => e.key === 'Escape' && onClose()}
  ></div>

  <!-- Panel -->
  <div class="absolute top-full right-0 mt-2 w-80 bg-bg-secondary border border-border-subtle rounded-lg shadow-xl z-50 overflow-hidden">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border-subtle">
      <span class="text-sm font-medium text-text-primary">Port Forwards</span>
      <button onclick={onClose} class="p-1 text-text-muted hover:text-text-primary transition-colors">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Active Forwards -->
    <div class="max-h-48 overflow-y-auto">
      {#if $portForwards.length === 0}
        <div class="py-6 text-center text-text-muted text-sm">
          No active forwards
        </div>
      {:else}
        {#each $portForwards as forward}
          <div class="flex items-center justify-between px-4 py-2.5 border-b border-border-subtle/50 hover:bg-bg-tertiary/30 transition-colors group">
            <div class="flex items-center gap-3 min-w-0 flex-1">
              <div class="w-1.5 h-1.5 rounded-full flex-shrink-0 {getStatusIndicator(forward.status)}"></div>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-1.5 text-sm">
                  <span class="font-mono text-accent-primary">{forward.local_port}</span>
                  <svg class="w-3 h-3 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
                  </svg>
                  <span class="font-mono text-text-secondary">{forward.remote_port}</span>
                </div>
                <div class="flex items-center gap-2 text-xs text-text-muted">
                  <span class="truncate">{forward.resource_type === 'service' ? 'svc/' : ''}{forward.resource_name}</span>
                  {#if forward.status === 'active' && (forward.total_connections > 0 || forward.bytes_sent > 0 || forward.bytes_received > 0)}
                    <span class="text-text-muted/60">·</span>
                    <span class="flex-shrink-0" title="{forward.active_connections} active / {forward.total_connections} total connections">
                      {forward.active_connections}/{forward.total_connections} conn
                    </span>
                    <span class="text-text-muted/60">·</span>
                    <span class="flex-shrink-0" title="Sent: {formatBytes(forward.bytes_sent)}, Received: {formatBytes(forward.bytes_received)}">
                      ↑{formatBytes(forward.bytes_sent)} ↓{formatBytes(forward.bytes_received)}
                    </span>
                  {/if}
                </div>
              </div>
            </div>
            {#if forward.status === 'active' || forward.status === 'starting'}
              <button
                onclick={() => handleStopForward(forward.id)}
                class="p-1.5 text-text-muted hover:text-accent-error hover:bg-accent-error/10 rounded transition-colors opacity-0 group-hover:opacity-100 flex-shrink-0"
                title="Stop"
              >
                <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
                  <rect x="6" y="6" width="12" height="12" rx="1" />
                </svg>
              </button>
            {/if}
          </div>
        {/each}
      {/if}
    </div>

    <!-- New Forward Form -->
    <div class="border-t border-border-subtle">
      {#if !showNewForm}
        <button
          onclick={openNewForm}
          class="w-full flex items-center justify-center gap-2 px-4 py-2.5 text-sm text-text-muted hover:text-accent-primary hover:bg-bg-tertiary/50 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          New Forward
        </button>
      {:else}
        <div class="p-4 space-y-3 bg-bg-tertiary/30">
          <!-- Error -->
          {#if formError}
            <div class="text-xs text-accent-error bg-accent-error/10 px-2 py-1.5 rounded">
              {formError}
            </div>
          {/if}

          <!-- Resource Type Toggle -->
          <div class="flex rounded-lg bg-bg-primary p-0.5 border border-border-subtle">
            <button
              onclick={() => { resourceType = 'pod'; selectedResource = ''; loadResourcesForNamespace(); }}
              class="flex-1 px-3 py-1.5 text-xs font-medium rounded-md transition-colors {resourceType === 'pod' ? 'bg-accent-primary text-white' : 'text-text-muted hover:text-text-primary'}"
            >
              Pod
            </button>
            <button
              onclick={() => { resourceType = 'service'; selectedResource = ''; loadResourcesForNamespace(); }}
              class="flex-1 px-3 py-1.5 text-xs font-medium rounded-md transition-colors {resourceType === 'service' ? 'bg-accent-primary text-white' : 'text-text-muted hover:text-text-primary'}"
            >
              Service
            </button>
          </div>

          <!-- Namespace -->
          <CustomSelect
            bind:value={selectedNamespaceForm}
            options={$namespaces.map(ns => ({ value: ns, label: ns }))}
            placeholder="Namespace..."
            onchange={() => { selectedResource = ''; loadResourcesForNamespace(); }}
          />

          <!-- Resource -->
          <CustomSelect
            bind:value={selectedResource}
            options={getAvailableResources().map(r => ({ value: r, label: r }))}
            placeholder={isLoadingResources ? 'Loading...' : `Select ${resourceType}...`}
            disabled={!selectedNamespaceForm || isLoadingResources}
            onchange={handleResourceChange}
          />

          <!-- Ports -->
          <div class="flex items-center gap-2">
            <div class="flex-1">
              <input
                type="number"
                bind:value={localPort}
                min="1"
                max="65535"
                placeholder="Local"
                class="w-full px-3 py-2 bg-bg-primary border border-border-subtle rounded-lg text-sm text-text-primary text-center font-mono focus:outline-none focus:border-accent-primary [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
              />
              <div class="text-xs text-text-muted text-center mt-1">Local</div>
            </div>
            <svg class="w-4 h-4 text-text-muted mt-[-16px]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
            </svg>
            <div class="flex-1">
              {#if availablePorts.length > 0}
                {@const portValue = String(remotePort)}
                <CustomSelect
                  value={portValue}
                  options={availablePorts.map(p => ({
                    value: String(p.port),
                    label: `${p.port}${p.name ? ` (${p.name})` : ''}`
                  }))}
                  placeholder="Port..."
                  onchange={(v) => { remotePort = Number(v); localPort = Number(v); }}
                />
              {:else}
                <input
                  type="number"
                  bind:value={remotePort}
                  min="1"
                  max="65535"
                  placeholder="Port"
                  disabled={isLoadingPorts}
                  class="w-full px-3 py-2 bg-bg-primary border border-border-subtle rounded-lg text-sm text-text-primary text-center font-mono focus:outline-none focus:border-accent-primary disabled:opacity-50 [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
                />
              {/if}
              <div class="text-xs text-text-muted text-center mt-1">
                {isLoadingPorts ? 'Loading...' : 'Remote'}
              </div>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex gap-2 pt-1">
            <button
              onclick={resetForm}
              class="flex-1 px-3 py-2 text-sm text-text-muted hover:text-text-primary transition-colors"
            >
              Cancel
            </button>
            <button
              onclick={handleStartForward}
              disabled={isStarting || !selectedResource || !selectedNamespaceForm || !remotePort}
              class="flex-1 px-3 py-2 text-sm bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
            >
              {isStarting ? 'Starting...' : 'Forward'}
            </button>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}
