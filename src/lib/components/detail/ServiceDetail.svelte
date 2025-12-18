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

  type TabType = 'overview' | 'endpoints' | 'events' | 'yaml';
  let activeTab = $state<TabType>('overview');

  // Service detail data
  interface ServiceDetail {
    name: string;
    namespace: string;
    uid: string;
    creation_timestamp: string;
    labels: Record<string, string>;
    annotations: Record<string, string>;
    service_type: string;
    cluster_ip: string | null;
    cluster_ips: string[];
    external_ips: string[];
    ports: ServicePort[];
    selector: Record<string, string>;
    session_affinity: string;
    load_balancer_ip: string | null;
    load_balancer_ingress: string[];
    external_name: string | null;
    internal_traffic_policy: string | null;
    external_traffic_policy: string | null;
  }

  interface ServicePort {
    name: string | null;
    protocol: string;
    port: number;
    target_port: string;
    node_port: number | null;
  }

  interface ServiceEndpoint {
    ip: string;
    port: number;
    protocol: string;
    target_name: string | null;
    ready: boolean;
  }

  interface ServiceEvent {
    event_type: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp: string | null;
    last_timestamp: string | null;
    source: string;
  }

  let serviceDetail = $state<ServiceDetail | null>(null);
  let serviceYaml = $state<string>('');
  let serviceEndpoints = $state<ServiceEndpoint[]>([]);
  let serviceEvents = $state<ServiceEvent[]>([]);
  let isLoading = $state<boolean>(true);
  let error = $state<string | null>(null);

  // Tombstone state
  let isDeleted = $state<boolean>(false);
  let lastSeen = $state<string | null>(null);
  let existenceInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await loadServiceDetail();
    existenceInterval = setInterval(checkServiceExists, 5000);

    return () => {
      if (existenceInterval) {
        clearInterval(existenceInterval);
        existenceInterval = null;
      }
    };
  });

  async function checkServiceExists() {
    if (isDeleted) return;

    try {
      await invoke<ServiceDetail>('get_service_detail', {
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

  async function loadServiceDetail() {
    try {
      isLoading = true;
      error = null;
      serviceDetail = await invoke<ServiceDetail>('get_service_detail', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      console.error('Failed to load service detail:', e);
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  async function loadServiceYaml() {
    try {
      serviceYaml = await invoke<string>('get_service_yaml', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      serviceYaml = `Error loading YAML: ${e}`;
    }
  }

  async function loadServiceEndpoints() {
    try {
      serviceEndpoints = await invoke<ServiceEndpoint[]>('get_service_endpoints', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      serviceEndpoints = [];
    }
  }

  async function loadServiceEvents() {
    try {
      serviceEvents = await invoke<ServiceEvent[]>('get_service_events', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      serviceEvents = [];
    }
  }

  function handleTabChange(tab: TabType) {
    activeTab = tab;

    if (tab === 'yaml' && !serviceYaml) {
      loadServiceYaml();
    } else if (tab === 'endpoints' && serviceEndpoints.length === 0) {
      loadServiceEndpoints();
    } else if (tab === 'events' && serviceEvents.length === 0) {
      loadServiceEvents();
    }
  }

  function getServiceTypeColor(type: string): string {
    switch (type) {
      case 'ClusterIP': return 'text-accent-primary bg-accent-primary/10';
      case 'NodePort': return 'text-accent-warning bg-accent-warning/10';
      case 'LoadBalancer': return 'text-accent-success bg-accent-success/10';
      case 'ExternalName': return 'text-purple-400 bg-purple-400/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
  }

  function getEventTypeColor(type: string): string {
    return type === 'Normal' ? 'text-accent-success bg-accent-success/10' : 'text-accent-warning bg-accent-warning/10';
  }

  function formatPort(port: ServicePort): string {
    let str = `${port.port}`;
    if (port.target_port && port.target_port !== String(port.port)) {
      str += `:${port.target_port}`;
    }
    if (port.node_port) {
      str += `:${port.node_port}`;
    }
    str += `/${port.protocol}`;
    if (port.name) {
      str = `${port.name} (${str})`;
    }
    return str;
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
        onclick={() => handleTabChange('endpoints')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'endpoints' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Endpoints
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
          <span class="font-medium text-accent-warning">This service no longer exists</span>
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
          <p class="text-text-muted">Loading service details...</p>
        </div>
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-accent-error mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-muted">{error}</p>
          <button onclick={loadServiceDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">
            Retry
          </button>
        </div>
      </div>
    {:else if !serviceDetail}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-text-muted mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M12 12h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-text-muted">No service data loaded</p>
          <button onclick={loadServiceDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">
            Retry
          </button>
        </div>
      </div>
    {:else if activeTab === 'overview'}
      <div class="p-6 space-y-6">
        <!-- Service Type & Basic Info -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Service Info</h2>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Type</div>
              <span class="text-sm px-2 py-0.5 rounded {getServiceTypeColor(serviceDetail.service_type)}">
                {serviceDetail.service_type}
              </span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Cluster IP</div>
              <span class="text-sm text-text-primary font-mono">{serviceDetail.cluster_ip || 'None'}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Session Affinity</div>
              <span class="text-sm text-text-primary">{serviceDetail.session_affinity}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Created</div>
              <span class="text-sm text-text-primary">{serviceDetail.creation_timestamp}</span>
            </div>
          </div>
        </section>

        <!-- Ports Section -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Ports</h2>
          {#if serviceDetail.ports.length > 0}
            <div class="bg-bg-secondary rounded-lg overflow-hidden">
              <table class="w-full text-sm">
                <thead class="bg-bg-tertiary">
                  <tr>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Name</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Port</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Target Port</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Node Port</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Protocol</th>
                  </tr>
                </thead>
                <tbody>
                  {#each serviceDetail.ports as port}
                    <tr class="border-t border-border-subtle">
                      <td class="px-4 py-2 text-text-primary">{port.name || '-'}</td>
                      <td class="px-4 py-2 text-accent-primary font-mono">{port.port}</td>
                      <td class="px-4 py-2 text-text-secondary font-mono">{port.target_port}</td>
                      <td class="px-4 py-2 text-text-secondary font-mono">{port.node_port || '-'}</td>
                      <td class="px-4 py-2 text-text-muted">{port.protocol}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="bg-bg-secondary rounded-lg p-4">
              <p class="text-text-muted text-sm">No ports defined</p>
            </div>
          {/if}
        </section>

        <!-- External Access -->
        {#if serviceDetail.service_type === 'LoadBalancer' || serviceDetail.external_ips.length > 0 || serviceDetail.external_name}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">External Access</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              {#if serviceDetail.load_balancer_ip}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Load Balancer IP</div>
                  <span class="text-sm text-accent-success font-mono">{serviceDetail.load_balancer_ip}</span>
                </div>
              {/if}
              {#if serviceDetail.load_balancer_ingress.length > 0}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Load Balancer Ingress</div>
                  <div class="space-y-1">
                    {#each serviceDetail.load_balancer_ingress as ingress}
                      <span class="block text-sm text-accent-success font-mono">{ingress}</span>
                    {/each}
                  </div>
                </div>
              {/if}
              {#if serviceDetail.external_ips.length > 0}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">External IPs</div>
                  <div class="space-y-1">
                    {#each serviceDetail.external_ips as ip}
                      <span class="block text-sm text-accent-primary font-mono">{ip}</span>
                    {/each}
                  </div>
                </div>
              {/if}
              {#if serviceDetail.external_name}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">External Name</div>
                  <span class="text-sm text-accent-primary font-mono">{serviceDetail.external_name}</span>
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Traffic Policies -->
        {#if serviceDetail.internal_traffic_policy || serviceDetail.external_traffic_policy}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Traffic Policies</h2>
            <div class="grid grid-cols-2 gap-4">
              {#if serviceDetail.internal_traffic_policy}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Internal Traffic</div>
                  <span class="text-sm text-text-primary">{serviceDetail.internal_traffic_policy}</span>
                </div>
              {/if}
              {#if serviceDetail.external_traffic_policy}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">External Traffic</div>
                  <span class="text-sm text-text-primary">{serviceDetail.external_traffic_policy}</span>
                </div>
              {/if}
            </div>
          </section>
        {/if}

        <!-- Selector -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Selector</h2>
          <div class="bg-bg-secondary rounded-lg p-4">
            {#if Object.keys(serviceDetail.selector).length > 0}
              <div class="flex flex-wrap gap-2">
                {#each Object.entries(serviceDetail.selector) as [key, value]}
                  <span class="text-xs bg-accent-primary/10 text-accent-primary px-2 py-1 rounded font-mono">
                    {key}={value}
                  </span>
                {/each}
              </div>
            {:else}
              <p class="text-text-muted text-sm">No selector (headless service or ExternalName)</p>
            {/if}
          </div>
        </section>

        <!-- Labels & Annotations -->
        <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Labels</h2>
            <div class="bg-bg-secondary rounded-lg p-4">
              {#if Object.keys(serviceDetail.labels).length > 0}
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(serviceDetail.labels) as [key, value]}
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
              {#if Object.keys(serviceDetail.annotations).length > 0}
                <div class="space-y-1">
                  {#each Object.entries(serviceDetail.annotations) as [key, value]}
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

    {:else if activeTab === 'endpoints'}
      <div class="p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-text-primary">Endpoints ({serviceEndpoints.length})</h2>
          <button
            onclick={loadServiceEndpoints}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        {#if serviceEndpoints.length > 0}
          <div class="bg-bg-secondary rounded-lg overflow-hidden">
            <table class="w-full text-sm">
              <thead class="bg-bg-tertiary">
                <tr>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">IP</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Port</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Protocol</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Target</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Ready</th>
                </tr>
              </thead>
              <tbody>
                {#each serviceEndpoints as endpoint}
                  <tr class="border-t border-border-subtle">
                    <td class="px-4 py-2 text-accent-primary font-mono">{endpoint.ip}</td>
                    <td class="px-4 py-2 text-text-primary font-mono">{endpoint.port}</td>
                    <td class="px-4 py-2 text-text-muted">{endpoint.protocol}</td>
                    <td class="px-4 py-2 text-text-secondary">{endpoint.target_name || '-'}</td>
                    <td class="px-4 py-2">
                      <span class="text-xs px-2 py-0.5 rounded {endpoint.ready ? 'text-accent-success bg-accent-success/10' : 'text-accent-error bg-accent-error/10'}">
                        {endpoint.ready ? 'Ready' : 'Not Ready'}
                      </span>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="flex items-center justify-center h-48">
            <p class="text-text-muted">No endpoints found</p>
          </div>
        {/if}
      </div>

    {:else if activeTab === 'events'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <span class="text-sm text-text-muted">Service Events</span>
          <button
            onclick={loadServiceEvents}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4">
          {#if serviceEvents.length > 0}
            <div class="space-y-3">
              {#each serviceEvents as event}
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
        yaml={serviceYaml}
        context={context}
        resourceType="Service"
        isDeleted={isDeleted}
        onRefresh={loadServiceYaml}
        onApplySuccess={loadServiceDetail}
      />
    {/if}
  </div>
</div>
