<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    context: string;
    namespace: string;
    name: string;
  }

  let { context, namespace, name }: Props = $props();

  type TabType = 'overview' | 'yaml' | 'logs' | 'events';
  let activeTab = $state<TabType>('overview');

  // Pod detail data
  interface PodDetail {
    name: string;
    namespace: string;
    uid: string;
    creation_timestamp: string;
    labels: Record<string, string>;
    annotations: Record<string, string>;
    status: string;
    phase: string;
    pod_ip: string | null;
    node_name: string | null;
    service_account: string | null;
    conditions: PodCondition[];
    containers: ContainerDetail[];
    init_containers: ContainerDetail[];
    volumes: VolumeInfo[];
  }

  interface PodCondition {
    type: string;
    status: string;
    reason: string | null;
    message: string | null;
    last_transition_time: string | null;
  }

  interface ContainerDetail {
    name: string;
    image: string;
    ready: boolean;
    restart_count: number;
    state: string;
    state_reason: string | null;
    started_at: string | null;
    ports: ContainerPort[];
    resources: ContainerResources;
    env_count: number;
    volume_mounts: string[];
  }

  interface ContainerPort {
    name: string | null;
    container_port: number;
    protocol: string;
  }

  interface ContainerResources {
    cpu_request: string | null;
    cpu_limit: string | null;
    memory_request: string | null;
    memory_limit: string | null;
  }

  interface VolumeInfo {
    name: string;
    volume_type: string;
  }

  interface PodEvent {
    type: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp: string | null;
    last_timestamp: string | null;
    source: string;
  }

  let podDetail = $state<PodDetail | null>(null);
  let podYaml = $state<string>('');
  let podLogs = $state<string>('');
  let podEvents = $state<PodEvent[]>([]);
  let isLoading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let selectedContainer = $state<string>('');
  let logsLoading = $state<boolean>(false);
  let autoRefreshLogs = $state<boolean>(false);
  let logsInterval: number | null = null;

  onMount(async () => {
    await loadPodDetail();
  });

  async function loadPodDetail() {
    try {
      isLoading = true;
      error = null;
      podDetail = await invoke<PodDetail>('get_pod_detail', {
        contextName: context,
        namespace,
        name
      });
      if (podDetail && podDetail.containers.length > 0) {
        selectedContainer = podDetail.containers[0].name;
      }
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  async function loadPodYaml() {
    try {
      podYaml = await invoke<string>('get_pod_yaml', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      podYaml = `Error loading YAML: ${e}`;
    }
  }

  async function loadPodLogs() {
    if (!selectedContainer) return;
    try {
      logsLoading = true;
      podLogs = await invoke<string>('get_pod_logs', {
        namespace,
        podName: name,
        container: selectedContainer,
        tailLines: 500
      });
    } catch (e) {
      podLogs = `Error loading logs: ${e}`;
    } finally {
      logsLoading = false;
    }
  }

  async function loadPodEvents() {
    try {
      podEvents = await invoke<PodEvent[]>('get_pod_events', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      podEvents = [];
    }
  }

  function handleTabChange(tab: TabType) {
    activeTab = tab;
    if (tab === 'yaml' && !podYaml) {
      loadPodYaml();
    } else if (tab === 'logs' && !podLogs) {
      loadPodLogs();
    } else if (tab === 'events' && podEvents.length === 0) {
      loadPodEvents();
    }
  }

  function toggleAutoRefresh() {
    autoRefreshLogs = !autoRefreshLogs;
    if (autoRefreshLogs) {
      logsInterval = setInterval(() => loadPodLogs(), 3000);
    } else if (logsInterval) {
      clearInterval(logsInterval);
      logsInterval = null;
    }
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case 'running': return 'text-accent-success bg-accent-success/10';
      case 'succeeded': return 'text-accent-success bg-accent-success/10';
      case 'pending': return 'text-accent-warning bg-accent-warning/10';
      case 'failed': return 'text-accent-error bg-accent-error/10';
      case 'terminated': return 'text-accent-error bg-accent-error/10';
      case 'waiting': return 'text-accent-warning bg-accent-warning/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
  }

  function getConditionColor(status: string): string {
    return status === 'True' ? 'text-accent-success' : 'text-accent-error';
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
        onclick={() => handleTabChange('yaml')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'yaml' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        YAML
      </button>
      <button
        onclick={() => handleTabChange('logs')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'logs' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Logs
      </button>
      <button
        onclick={() => handleTabChange('events')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'events' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Events
      </button>
    </nav>
  </div>

  <!-- Tab content -->
  <div class="flex-1 overflow-auto">
    {#if isLoading}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-8 h-8 text-accent-primary mx-auto mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <p class="text-text-muted">Loading pod details...</p>
        </div>
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-accent-error mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-muted">{error}</p>
          <button onclick={loadPodDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">
            Retry
          </button>
        </div>
      </div>
    {:else if activeTab === 'overview' && podDetail}
      <div class="p-6 space-y-6">
        <!-- Status Section -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Status</h2>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Phase</div>
              <span class="text-sm px-2 py-0.5 rounded {getStatusColor(podDetail.phase)}">{podDetail.phase}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Pod IP</div>
              <code class="text-sm text-text-primary">{podDetail.pod_ip || 'N/A'}</code>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Node</div>
              <span class="text-sm text-text-primary">{podDetail.node_name || 'N/A'}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Service Account</div>
              <span class="text-sm text-text-primary">{podDetail.service_account || 'default'}</span>
            </div>
          </div>
        </section>

        <!-- Conditions Section -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Conditions</h2>
          <div class="bg-bg-secondary rounded-lg overflow-hidden">
            <table class="w-full text-sm">
              <thead class="bg-bg-tertiary">
                <tr>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Type</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Status</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Reason</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Last Transition</th>
                </tr>
              </thead>
              <tbody>
                {#each podDetail.conditions as condition}
                  <tr class="border-t border-border-subtle">
                    <td class="px-4 py-2 text-text-primary">{condition.type}</td>
                    <td class="px-4 py-2 {getConditionColor(condition.status)}">{condition.status}</td>
                    <td class="px-4 py-2 text-text-secondary">{condition.reason || '-'}</td>
                    <td class="px-4 py-2 text-text-muted">{condition.last_transition_time || '-'}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </section>

        <!-- Containers Section -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Containers ({podDetail.containers.length})</h2>
          <div class="space-y-4">
            {#each podDetail.containers as container}
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="flex items-center justify-between mb-3">
                  <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full {container.ready ? 'bg-accent-success' : 'bg-accent-error'}"></div>
                    <span class="font-medium text-text-primary">{container.name}</span>
                    <span class="text-xs px-2 py-0.5 rounded {getStatusColor(container.state)}">{container.state}</span>
                  </div>
                  <span class="text-xs text-text-muted">Restarts: {container.restart_count}</span>
                </div>
                <div class="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span class="text-text-muted">Image:</span>
                    <code class="ml-2 text-text-secondary break-all">{container.image}</code>
                  </div>
                  {#if container.ports.length > 0}
                    <div>
                      <span class="text-text-muted">Ports:</span>
                      <span class="ml-2 text-text-secondary">
                        {container.ports.map(p => `${p.container_port}/${p.protocol}`).join(', ')}
                      </span>
                    </div>
                  {/if}
                  {#if container.resources.cpu_request || container.resources.memory_request}
                    <div>
                      <span class="text-text-muted">Resources:</span>
                      <span class="ml-2 text-text-secondary">
                        CPU: {container.resources.cpu_request || '-'}/{container.resources.cpu_limit || '-'},
                        Mem: {container.resources.memory_request || '-'}/{container.resources.memory_limit || '-'}
                      </span>
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </section>

        <!-- Labels & Annotations -->
        <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Labels</h2>
            <div class="bg-bg-secondary rounded-lg p-4">
              {#if Object.keys(podDetail.labels).length > 0}
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(podDetail.labels) as [key, value]}
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
              {#if Object.keys(podDetail.annotations).length > 0}
                <div class="space-y-1">
                  {#each Object.entries(podDetail.annotations) as [key, value]}
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

        <!-- Volumes -->
        {#if podDetail.volumes.length > 0}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Volumes ({podDetail.volumes.length})</h2>
            <div class="bg-bg-secondary rounded-lg overflow-hidden">
              <table class="w-full text-sm">
                <thead class="bg-bg-tertiary">
                  <tr>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Name</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Type</th>
                  </tr>
                </thead>
                <tbody>
                  {#each podDetail.volumes as volume}
                    <tr class="border-t border-border-subtle">
                      <td class="px-4 py-2 text-text-primary">{volume.name}</td>
                      <td class="px-4 py-2 text-text-secondary">{volume.volume_type}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </section>
        {/if}
      </div>

    {:else if activeTab === 'yaml'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <span class="text-sm text-text-muted">Pod YAML Manifest</span>
          <button
            onclick={loadPodYaml}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors"
          >
            Refresh
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4">
          <pre class="text-xs font-mono text-text-secondary whitespace-pre-wrap bg-bg-secondary p-4 rounded-lg">{podYaml || 'Loading...'}</pre>
        </div>
      </div>

    {:else if activeTab === 'logs'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <div class="flex items-center gap-4">
            <select
              bind:value={selectedContainer}
              onchange={loadPodLogs}
              class="text-sm bg-bg-tertiary border border-border-subtle rounded px-3 py-1 text-text-primary"
            >
              {#if podDetail}
                {#each podDetail.containers as container}
                  <option value={container.name}>{container.name}</option>
                {/each}
              {/if}
            </select>
            <label class="flex items-center gap-2 text-sm text-text-muted">
              <input type="checkbox" bind:checked={autoRefreshLogs} onchange={toggleAutoRefresh} class="rounded" />
              Auto-refresh (3s)
            </label>
          </div>
          <button
            onclick={loadPodLogs}
            disabled={logsLoading}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            {logsLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4 bg-[#1e1e1e]">
          <pre class="text-xs font-mono text-green-400 whitespace-pre-wrap">{podLogs || 'No logs available'}</pre>
        </div>
      </div>

    {:else if activeTab === 'events'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <span class="text-sm text-text-muted">Pod Events</span>
          <button
            onclick={loadPodEvents}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors"
          >
            Refresh
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4">
          {#if podEvents.length > 0}
            <div class="space-y-3">
              {#each podEvents as event}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="flex items-start justify-between mb-2">
                    <div class="flex items-center gap-2">
                      <span class="text-xs px-2 py-0.5 rounded {getEventTypeColor(event.type)}">{event.type}</span>
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
    {/if}
  </div>
</div>
