<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import YamlEditor from '../ui/YamlEditor.svelte';

  interface Props {
    context: string;
    namespace: string;
    name: string;
  }

  let { context, namespace, name }: Props = $props();

  type TabType = 'overview' | 'containers' | 'env' | 'volumes' | 'events' | 'logs' | 'yaml';
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

  interface EnvVarInfo {
    name: string;
    value: string | null;
    source: string; // "literal", "configMapKeyRef", "secretKeyRef", "fieldRef", "resourceFieldRef"
    source_name: string | null;
    source_key: string | null;
  }

  interface VolumeMountInfo {
    name: string;
    mount_path: string;
    read_only: boolean;
    sub_path: string | null;
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
    env_vars: EnvVarInfo[];
    volume_mounts: VolumeMountInfo[];
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
    source_name: string | null;
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
  let autoRefreshLogs = $state<boolean>(true); // Tailing enabled by default
  let logsInterval: ReturnType<typeof setInterval> | null = null;

  // Tombstone state - when pod is deleted while window is open
  let isDeleted = $state<boolean>(false);
  let lastSeen = $state<string | null>(null);
  let existenceInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await loadPodDetail();

    // Start polling for pod existence every 5 seconds
    existenceInterval = setInterval(checkPodExists, 5000);

    // Cleanup intervals on unmount
    return () => {
      if (logsInterval) {
        clearInterval(logsInterval);
        logsInterval = null;
      }
      if (existenceInterval) {
        clearInterval(existenceInterval);
        existenceInterval = null;
      }
    };
  });

  async function checkPodExists() {
    if (isDeleted) return; // Already marked as deleted

    try {
      await invoke<PodDetail>('get_pod_detail', {
        contextName: context,
        namespace,
        name
      });
      // Pod still exists, update lastSeen
      lastSeen = new Date().toLocaleTimeString();
    } catch (e) {
      const errorStr = String(e);
      // Check if it's a "not found" error
      if (errorStr.toLowerCase().includes('not found') ||
          errorStr.toLowerCase().includes('404') ||
          errorStr.toLowerCase().includes('does not exist')) {
        isDeleted = true;
        // Stop checking once we know it's deleted
        if (existenceInterval) {
          clearInterval(existenceInterval);
          existenceInterval = null;
        }
        // Stop log tailing
        if (logsInterval) {
          clearInterval(logsInterval);
          logsInterval = null;
        }
      }
    }
  }

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
    // Stop tailing when leaving logs tab
    if (activeTab === 'logs' && tab !== 'logs' && logsInterval) {
      clearInterval(logsInterval);
      logsInterval = null;
    }

    activeTab = tab;

    if (tab === 'yaml' && !podYaml) {
      loadPodYaml();
    } else if (tab === 'logs') {
      loadPodLogs();
      // Start tailing if enabled
      if (autoRefreshLogs && !logsInterval) {
        logsInterval = setInterval(() => loadPodLogs(), 3000);
      }
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

  async function openTerminal(containerName?: string) {
    try {
      await invoke('open_terminal_window', {
        podName: name,
        namespace,
        context,
        container: containerName || null,
      });
    } catch (e) {
      console.error('Failed to open terminal:', e);
    }
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
        onclick={() => handleTabChange('containers')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'containers' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Containers
      </button>
      <button
        onclick={() => handleTabChange('env')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'env' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Env
      </button>
      <button
        onclick={() => handleTabChange('volumes')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'volumes' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Volumes
      </button>
      <button
        onclick={() => handleTabChange('events')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'events' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Events
      </button>
      <button
        onclick={() => handleTabChange('logs')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'logs' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Logs
      </button>
      <button
        onclick={() => handleTabChange('yaml')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'yaml' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        YAML
      </button>
    </nav>
  </div>

  <!-- Tombstone Banner - Pod deleted while window open -->
  {#if isDeleted}
    <div class="flex items-center justify-between px-4 py-3 bg-accent-warning/10 border-b border-accent-warning/30">
      <div class="flex items-center gap-3">
        <svg class="w-5 h-5 text-accent-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div>
          <span class="font-medium text-accent-warning">This pod no longer exists</span>
          {#if lastSeen}
            <span class="text-text-muted text-sm ml-2">Last seen: {lastSeen}</span>
          {/if}
        </div>
      </div>
      <span class="text-xs text-text-muted bg-bg-tertiary px-2 py-1 rounded">Read-only - Actions disabled</span>
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
      </div>

    {:else if activeTab === 'containers' && podDetail}
      <div class="p-6 space-y-6">
        <!-- Init Containers -->
        {#if podDetail.init_containers.length > 0}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Init Containers ({podDetail.init_containers.length})</h2>
            <div class="space-y-4">
              {#each podDetail.init_containers as container}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="flex items-center justify-between mb-3">
                    <div class="flex items-center gap-2">
                      <div class="w-2 h-2 rounded-full {container.ready ? 'bg-accent-success' : 'bg-accent-error'}"></div>
                      <span class="font-medium text-text-primary">{container.name}</span>
                      <span class="text-xs px-2 py-0.5 rounded bg-bg-tertiary text-text-muted">init</span>
                      <span class="text-xs px-2 py-0.5 rounded {getStatusColor(container.state)}">{container.state}</span>
                    </div>
                    <div class="flex items-center gap-3">
                      <span class="text-xs text-text-muted">Restarts: {container.restart_count}</span>
                      <button
                        onclick={() => openTerminal(container.name)}
                        disabled={isDeleted}
                        class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-primary transition-colors disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-text-muted"
                        title={isDeleted ? "Pod no longer exists" : `Open Terminal in ${container.name}`}
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                        </svg>
                      </button>
                    </div>
                  </div>
                  <div class="text-sm">
                    <span class="text-text-muted">Image:</span>
                    <code class="ml-2 text-text-secondary break-all">{container.image}</code>
                  </div>
                </div>
              {/each}
            </div>
          </section>
        {/if}

        <!-- Containers -->
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
                  <div class="flex items-center gap-3">
                    <span class="text-xs text-text-muted">Restarts: {container.restart_count}</span>
                    <button
                      onclick={() => openTerminal(container.name)}
                      disabled={isDeleted}
                      class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-primary transition-colors disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-transparent disabled:hover:text-text-muted"
                      title={isDeleted ? "Pod no longer exists" : `Open Terminal in ${container.name}`}
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                      </svg>
                    </button>
                  </div>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
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
                  <div>
                    <span class="text-text-muted">CPU:</span>
                    <span class="ml-2 text-text-secondary">
                      {container.resources.cpu_request || '-'} / {container.resources.cpu_limit || '-'}
                    </span>
                  </div>
                  <div>
                    <span class="text-text-muted">Memory:</span>
                    <span class="ml-2 text-text-secondary">
                      {container.resources.memory_request || '-'} / {container.resources.memory_limit || '-'}
                    </span>
                  </div>
                </div>
                {#if container.state_reason}
                  <div class="mt-3 text-sm">
                    <span class="text-text-muted">Reason:</span>
                    <span class="ml-2 text-accent-warning">{container.state_reason}</span>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        </section>
      </div>

    {:else if activeTab === 'env' && podDetail}
      <div class="p-6">
        <div class="space-y-6">
          {#each [...podDetail.containers, ...podDetail.init_containers] as container}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4 flex items-center gap-2">
                <span>{container.name}</span>
                <span class="text-xs px-2 py-0.5 rounded bg-bg-tertiary text-text-muted">{container.env_vars.length} vars</span>
              </h2>
              {#if container.env_vars.length > 0}
                <div class="bg-bg-secondary rounded-lg overflow-hidden">
                  <table class="w-full text-sm">
                    <thead class="bg-bg-tertiary">
                      <tr>
                        <th class="px-4 py-2 text-left text-xs text-text-muted uppercase w-1/4">Name</th>
                        <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Value / Source</th>
                        <th class="px-4 py-2 text-left text-xs text-text-muted uppercase w-24">Type</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#each container.env_vars as env}
                        <tr class="border-t border-border-subtle">
                          <td class="px-4 py-2 text-text-primary font-mono text-xs">{env.name}</td>
                          <td class="px-4 py-2">
                            {#if env.source === 'literal'}
                              <code class="text-xs text-text-secondary break-all">{env.value || ''}</code>
                            {:else if env.source === 'secretKeyRef'}
                              <span class="text-xs text-accent-warning">
                                {env.source_name || '?'} → {env.source_key || '?'}
                              </span>
                            {:else if env.source === 'configMapKeyRef'}
                              <span class="text-xs text-accent-primary">
                                {env.source_name || '?'} → {env.source_key || '?'}
                              </span>
                            {:else if env.source === 'fieldRef'}
                              <span class="text-xs text-text-secondary">{env.source_name || '?'}</span>
                            {:else}
                              <span class="text-xs text-text-muted">{env.source_name || '-'}</span>
                            {/if}
                          </td>
                          <td class="px-4 py-2">
                            <span class="text-xs px-2 py-0.5 rounded {
                              env.source === 'secretKeyRef' ? 'bg-accent-warning/10 text-accent-warning' :
                              env.source === 'configMapKeyRef' ? 'bg-accent-primary/10 text-accent-primary' :
                              env.source === 'fieldRef' ? 'bg-bg-tertiary text-text-muted' :
                              'bg-bg-tertiary text-text-secondary'
                            }">
                              {env.source === 'literal' ? 'value' : env.source.replace('KeyRef', '')}
                            </span>
                          </td>
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              {:else}
                <p class="text-text-muted text-sm bg-bg-secondary rounded-lg p-4">No environment variables</p>
              {/if}
            </section>
          {/each}
        </div>
      </div>

    {:else if activeTab === 'volumes' && podDetail}
      <div class="p-6 space-y-6">
        <!-- Volume Definitions -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Volumes ({podDetail.volumes.length})</h2>
          {#if podDetail.volumes.length > 0}
            <div class="bg-bg-secondary rounded-lg overflow-hidden">
              <table class="w-full text-sm">
                <thead class="bg-bg-tertiary">
                  <tr>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Name</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Type</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Source</th>
                  </tr>
                </thead>
                <tbody>
                  {#each podDetail.volumes as volume}
                    <tr class="border-t border-border-subtle">
                      <td class="px-4 py-2 text-text-primary font-medium">{volume.name}</td>
                      <td class="px-4 py-2">
                        <span class="text-xs px-2 py-0.5 rounded {
                          volume.volume_type === 'Secret' ? 'bg-accent-warning/10 text-accent-warning' :
                          volume.volume_type === 'ConfigMap' ? 'bg-accent-primary/10 text-accent-primary' :
                          volume.volume_type === 'PersistentVolumeClaim' ? 'bg-accent-success/10 text-accent-success' :
                          'bg-bg-tertiary text-text-secondary'
                        }">
                          {volume.volume_type}
                        </span>
                      </td>
                      <td class="px-4 py-2 text-text-secondary text-sm">{volume.source_name || '-'}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <p class="text-text-muted text-sm bg-bg-secondary rounded-lg p-4">No volumes defined</p>
          {/if}
        </section>

        <!-- Volume Mounts by Container -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Volume Mounts</h2>
          <div class="space-y-4">
            {#each [...podDetail.containers, ...podDetail.init_containers] as container}
              {#if container.volume_mounts.length > 0}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <h3 class="text-sm font-medium text-text-primary mb-3">{container.name}</h3>
                  <div class="space-y-2">
                    {#each container.volume_mounts as mount}
                      <div class="flex items-center gap-4 text-sm">
                        <code class="text-accent-primary">{mount.name}</code>
                        <span class="text-text-muted">→</span>
                        <code class="text-text-secondary">{mount.mount_path}</code>
                        {#if mount.read_only}
                          <span class="text-xs px-2 py-0.5 rounded bg-accent-warning/10 text-accent-warning">read-only</span>
                        {/if}
                        {#if mount.sub_path}
                          <span class="text-xs text-text-muted">subPath: {mount.sub_path}</span>
                        {/if}
                      </div>
                    {/each}
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        </section>
      </div>

    {:else if activeTab === 'yaml'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <div class="flex items-center gap-2">
            <span class="text-sm text-text-muted">Pod YAML Manifest</span>
            {#if isDeleted}
              <span class="text-xs text-accent-warning">(last known state)</span>
            {/if}
          </div>
          <div class="flex items-center gap-2">
            <button
              onclick={() => navigator.clipboard.writeText(podYaml)}
              class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors"
              title="Copy to clipboard"
            >
              Copy
            </button>
            <button
              onclick={loadPodYaml}
              disabled={isDeleted}
              class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Refresh
            </button>
          </div>
        </div>
        <div class="flex-1 overflow-hidden">
          {#if podYaml}
            <YamlEditor content={podYaml} readonly={true} />
          {:else}
            <div class="flex items-center justify-center h-full text-text-muted">
              Loading YAML...
            </div>
          {/if}
        </div>
      </div>

    {:else if activeTab === 'logs'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <div class="flex items-center gap-4">
            <select
              bind:value={selectedContainer}
              onchange={loadPodLogs}
              disabled={isDeleted}
              class="text-sm bg-bg-tertiary border border-border-subtle rounded px-3 py-1 text-text-primary disabled:opacity-50"
            >
              {#if podDetail}
                {#each podDetail.containers as container}
                  <option value={container.name}>{container.name}</option>
                {/each}
              {/if}
            </select>
            {#if !isDeleted}
              <label class="flex items-center gap-2 text-sm text-text-muted">
                <input type="checkbox" bind:checked={autoRefreshLogs} onchange={toggleAutoRefresh} class="rounded" />
                Auto-refresh (3s)
              </label>
            {:else}
              <span class="text-xs text-accent-warning">Showing cached logs</span>
            {/if}
          </div>
          <button
            onclick={loadPodLogs}
            disabled={logsLoading || isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {logsLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4 bg-[#1e1e1e]">
          <pre class="text-xs font-mono text-green-400 whitespace-pre-wrap">{podLogs || (isDeleted ? 'No cached logs available' : 'No logs available')}</pre>
        </div>
      </div>

    {:else if activeTab === 'events'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <div class="flex items-center gap-2">
            <span class="text-sm text-text-muted">Pod Events</span>
            {#if isDeleted}
              <span class="text-xs text-accent-warning">(cached)</span>
            {/if}
          </div>
          <button
            onclick={loadPodEvents}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
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
