<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import YamlEditorPanel from '../ui/YamlEditorPanel.svelte';

  interface Props {
    context: string;
    namespace: string;
    name: string;
  }

  let { context, namespace, name }: Props = $props();

  type TabType = 'overview' | 'events' | 'yaml';
  let activeTab = $state<TabType>('overview');

  // Ingress detail data
  interface IngressDetail {
    name: string;
    namespace: string;
    uid: string;
    creation_timestamp: string;
    labels: Record<string, string>;
    annotations: Record<string, string>;
    ingress_class: string | null;
    rules: IngressRule[];
    tls: IngressTls[];
    default_backend: IngressBackend | null;
    load_balancer_addresses: string[];
  }

  interface IngressRule {
    host: string | null;
    paths: IngressPath[];
  }

  interface IngressPath {
    path: string;
    path_type: string;
    backend_service: string | null;
    backend_port: string | null;
  }

  interface IngressTls {
    hosts: string[];
    secret_name: string | null;
  }

  interface IngressBackend {
    service_name: string;
    service_port: string;
  }

  interface IngressEvent {
    event_type: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp: string | null;
    last_timestamp: string | null;
    source: string;
  }

  let ingressDetail = $state<IngressDetail | null>(null);
  let ingressYaml = $state<string>('');
  let ingressEvents = $state<IngressEvent[]>([]);
  let isLoading = $state<boolean>(true);
  let error = $state<string | null>(null);

  // Tombstone state
  let isDeleted = $state<boolean>(false);
  let lastSeen = $state<string | null>(null);
  let existenceInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await loadIngressDetail();
    existenceInterval = setInterval(checkIngressExists, 5000);

    return () => {
      if (existenceInterval) {
        clearInterval(existenceInterval);
        existenceInterval = null;
      }
    };
  });

  async function checkIngressExists() {
    if (isDeleted) return;

    try {
      await invoke<IngressDetail>('get_ingress_detail', {
        contextName: context,
        namespace,
        name
      });
      lastSeen = new Date().toLocaleTimeString();
    } catch (e) {
      const errorStr = String(e);
      if (errorStr.toLowerCase().includes('not found') ||
          errorStr.toLowerCase().includes('404')) {
        isDeleted = true;
        if (existenceInterval) {
          clearInterval(existenceInterval);
          existenceInterval = null;
        }
      }
    }
  }

  async function loadIngressDetail() {
    try {
      isLoading = true;
      error = null;
      ingressDetail = await invoke<IngressDetail>('get_ingress_detail', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      console.error('Failed to load ingress detail:', e);
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  async function loadIngressYaml() {
    try {
      ingressYaml = await invoke<string>('get_ingress_yaml', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      ingressYaml = `Error loading YAML: ${e}`;
    }
  }

  async function loadIngressEvents() {
    try {
      ingressEvents = await invoke<IngressEvent[]>('get_ingress_events', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      ingressEvents = [];
    }
  }

  function handleTabChange(tab: TabType) {
    activeTab = tab;

    if (tab === 'yaml' && !ingressYaml) {
      loadIngressYaml();
    } else if (tab === 'events' && ingressEvents.length === 0) {
      loadIngressEvents();
    }
  }

  function getEventTypeColor(type: string): string {
    return type === 'Normal' ? 'text-accent-success bg-accent-success/10' : 'text-accent-warning bg-accent-warning/10';
  }
</script>

<div class="h-full flex flex-col">
  <!-- Tabs -->
  <div class="border-b border-border-subtle bg-bg-secondary">
    <nav class="flex gap-1 px-4">
      <button
        onclick={() => handleTabChange('overview')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'overview' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Overview
      </button>
      <button
        onclick={() => handleTabChange('events')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'events' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Events
      </button>
      <button
        onclick={() => handleTabChange('yaml')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'yaml' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        YAML
      </button>
    </nav>
  </div>

  <!-- Tombstone Banner -->
  {#if isDeleted}
    <div class="flex items-center justify-between px-4 py-3 bg-accent-warning/10 border-b border-accent-warning/30">
      <div class="flex items-center gap-3">
        <svg class="w-5 h-5 text-accent-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div>
          <span class="font-medium text-accent-warning">This ingress no longer exists</span>
          {#if lastSeen}
            <span class="text-text-muted text-sm ml-2">Last seen: {lastSeen}</span>
          {/if}
        </div>
      </div>
      <span class="text-xs text-text-muted bg-bg-tertiary px-2 py-1 rounded">Read-only</span>
    </div>
  {/if}

  <!-- Tab content -->
  <div class="flex-1 overflow-auto">
    {#if isLoading}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-8 h-8 text-accent-primary mx-auto mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <p class="text-text-muted">Loading ingress details...</p>
        </div>
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-accent-error mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-muted">{error}</p>
          <button onclick={loadIngressDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">
            Retry
          </button>
        </div>
      </div>
    {:else if !ingressDetail}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-text-muted mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M12 12h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-text-muted">No ingress data loaded</p>
          <button onclick={loadIngressDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">
            Retry
          </button>
        </div>
      </div>
    {:else if activeTab === 'overview'}
      <div class="p-6 space-y-6">
        <!-- Basic Info -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Ingress Info</h2>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Ingress Class</div>
              <span class="text-sm text-accent-primary">{ingressDetail.ingress_class || 'None'}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Rules</div>
              <span class="text-lg font-semibold text-text-primary">{ingressDetail.rules.length}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">TLS</div>
              <span class="text-sm {ingressDetail.tls.length > 0 ? 'text-accent-success' : 'text-text-muted'}">
                {ingressDetail.tls.length > 0 ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Created</div>
              <span class="text-sm text-text-primary">{ingressDetail.creation_timestamp}</span>
            </div>
          </div>
        </section>

        <!-- Load Balancer Addresses -->
        {#if ingressDetail.load_balancer_addresses.length > 0}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Load Balancer Addresses</h2>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="flex flex-wrap gap-2">
                {#each ingressDetail.load_balancer_addresses as address}
                  <span class="text-sm text-accent-success font-mono bg-accent-success/10 px-2 py-1 rounded">{address}</span>
                {/each}
              </div>
            </div>
          </section>
        {/if}

        <!-- Rules Section -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Routing Rules</h2>
          {#if ingressDetail.rules.length > 0}
            <div class="space-y-4">
              {#each ingressDetail.rules as rule, idx}
                <div class="bg-bg-secondary rounded-lg overflow-hidden">
                  <div class="px-4 py-2 bg-bg-tertiary border-b border-border-subtle">
                    <span class="text-sm font-medium text-text-primary">
                      {rule.host || '*'} (Rule {idx + 1})
                    </span>
                  </div>
                  <div class="p-4">
                    <table class="w-full text-sm">
                      <thead>
                        <tr>
                          <th class="text-left text-xs text-text-muted uppercase pb-2">Path</th>
                          <th class="text-left text-xs text-text-muted uppercase pb-2">Type</th>
                          <th class="text-left text-xs text-text-muted uppercase pb-2">Backend Service</th>
                          <th class="text-left text-xs text-text-muted uppercase pb-2">Port</th>
                        </tr>
                      </thead>
                      <tbody>
                        {#each rule.paths as path}
                          <tr class="border-t border-border-subtle/50">
                            <td class="py-2 text-accent-primary font-mono">{path.path}</td>
                            <td class="py-2 text-text-muted">{path.path_type}</td>
                            <td class="py-2 text-text-primary">{path.backend_service || '-'}</td>
                            <td class="py-2 text-text-secondary font-mono">{path.backend_port || '-'}</td>
                          </tr>
                        {/each}
                      </tbody>
                    </table>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="bg-bg-secondary rounded-lg p-4">
              <p class="text-text-muted text-sm">No rules defined</p>
            </div>
          {/if}
        </section>

        <!-- TLS Section -->
        {#if ingressDetail.tls.length > 0}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">TLS Configuration</h2>
            <div class="bg-bg-secondary rounded-lg overflow-hidden">
              <table class="w-full text-sm">
                <thead class="bg-bg-tertiary">
                  <tr>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Hosts</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Secret</th>
                  </tr>
                </thead>
                <tbody>
                  {#each ingressDetail.tls as tls}
                    <tr class="border-t border-border-subtle">
                      <td class="px-4 py-2">
                        <div class="flex flex-wrap gap-1">
                          {#each tls.hosts as host}
                            <span class="text-xs bg-accent-primary/10 text-accent-primary px-2 py-0.5 rounded">{host}</span>
                          {/each}
                        </div>
                      </td>
                      <td class="px-4 py-2 text-text-secondary font-mono">{tls.secret_name || '-'}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </section>
        {/if}

        <!-- Default Backend -->
        {#if ingressDetail.default_backend}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Default Backend</h2>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="grid grid-cols-2 gap-4">
                <div>
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Service</div>
                  <span class="text-sm text-text-primary">{ingressDetail.default_backend.service_name}</span>
                </div>
                <div>
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Port</div>
                  <span class="text-sm text-text-secondary font-mono">{ingressDetail.default_backend.service_port}</span>
                </div>
              </div>
            </div>
          </section>
        {/if}

        <!-- Labels & Annotations -->
        <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Labels</h2>
            <div class="bg-bg-secondary rounded-lg p-4">
              {#if Object.keys(ingressDetail.labels).length > 0}
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(ingressDetail.labels) as [key, value]}
                    <span class="text-xs bg-bg-tertiary px-2 py-1 rounded text-text-secondary">
                      {key}={value}
                    </span>
                  {/each}
                </div>
              {:else}
                <p class="text-text-muted text-sm">No labels</p>
              {/if}
            </div>
          </div>
          <div>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Annotations</h2>
            <div class="bg-bg-secondary rounded-lg p-4 max-h-48 overflow-auto">
              {#if Object.keys(ingressDetail.annotations).length > 0}
                <div class="space-y-1">
                  {#each Object.entries(ingressDetail.annotations) as [key, value]}
                    <div class="text-xs">
                      <span class="text-text-muted">{key}:</span>
                      <span class="text-text-secondary ml-1 break-all">{value}</span>
                    </div>
                  {/each}
                </div>
              {:else}
                <p class="text-text-muted text-sm">No annotations</p>
              {/if}
            </div>
          </div>
        </section>
      </div>

    {:else if activeTab === 'events'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <span class="text-sm text-text-muted">Ingress Events</span>
          <button
            onclick={loadIngressEvents}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4">
          {#if ingressEvents.length > 0}
            <div class="space-y-3">
              {#each ingressEvents as event}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="flex items-start justify-between mb-2">
                    <div class="flex items-center gap-2">
                      <span class="text-xs px-2 py-0.5 rounded {getEventTypeColor(event.event_type)}">{event.event_type}</span>
                      <span class="font-medium text-text-primary">{event.reason}</span>
                      {#if event.count > 1}
                        <span class="text-xs text-text-muted">x{event.count}</span>
                      {/if}
                    </div>
                    <span class="text-xs text-text-muted">{event.last_timestamp || event.first_timestamp}</span>
                  </div>
                  <p class="text-sm text-text-secondary">{event.message}</p>
                  <p class="text-xs text-text-muted mt-1">Source: {event.source}</p>
                </div>
              {/each}
            </div>
          {:else}
            <div class="flex items-center justify-center h-full">
              <p class="text-text-muted">No events found</p>
            </div>
          {/if}
        </div>
      </div>

    {:else if activeTab === 'yaml'}
      <YamlEditorPanel
        yaml={ingressYaml}
        context={context}
        resourceType="Ingress"
        isDeleted={isDeleted}
        onRefresh={loadIngressYaml}
        onApplySuccess={loadIngressDetail}
      />
    {/if}
  </div>
</div>
