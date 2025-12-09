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

  type TabType = 'overview' | 'pods' | 'events' | 'yaml';
  let activeTab = $state<TabType>('overview');

  // Deployment detail data
  interface DeploymentDetail {
    name: string;
    namespace: string;
    uid: string;
    creation_timestamp: string;
    labels: Record<string, string>;
    annotations: Record<string, string>;
    replicas: number;
    ready_replicas: number;
    updated_replicas: number;
    available_replicas: number;
    strategy: string;
    min_ready_seconds: number;
    revision_history_limit: number | null;
    selector: Record<string, string>;
    conditions: DeploymentCondition[];
    container_images: string[];
  }

  interface DeploymentCondition {
    condition_type: string;
    status: string;
    reason: string | null;
    message: string | null;
    last_update_time: string | null;
    last_transition_time: string | null;
  }

  interface DeploymentEvent {
    event_type: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp: string | null;
    last_timestamp: string | null;
    source: string;
  }

  interface PodInfo {
    name: string;
    namespace: string;
    status: string;
    ready: string;
    restarts: number;
    age: string;
    node: string | null;
    ip: string | null;
  }

  let deploymentDetail = $state<DeploymentDetail | null>(null);
  let deploymentYaml = $state<string>('');
  let deploymentPods = $state<PodInfo[]>([]);
  let deploymentEvents = $state<DeploymentEvent[]>([]);
  let isLoading = $state<boolean>(true);
  let error = $state<string | null>(null);

  // Scale modal state
  let showScaleModal = $state<boolean>(false);
  let scaleReplicas = $state<number>(0);
  let isScaling = $state<boolean>(false);

  // Restart state
  let isRestarting = $state<boolean>(false);

  // Tombstone state
  let isDeleted = $state<boolean>(false);
  let lastSeen = $state<string | null>(null);
  let existenceInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await loadDeploymentDetail();
    existenceInterval = setInterval(checkDeploymentExists, 5000);

    return () => {
      if (existenceInterval) {
        clearInterval(existenceInterval);
        existenceInterval = null;
      }
    };
  });

  async function checkDeploymentExists() {
    if (isDeleted) return;

    try {
      await invoke<DeploymentDetail>('get_deployment_detail', {
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

  async function loadDeploymentDetail() {
    try {
      isLoading = true;
      error = null;
      console.log('Loading deployment detail:', { context, namespace, name });
      deploymentDetail = await invoke<DeploymentDetail>('get_deployment_detail', {
        contextName: context,
        namespace,
        name
      });
      console.log('Loaded deployment detail:', deploymentDetail);
      if (deploymentDetail) {
        scaleReplicas = deploymentDetail.replicas;
      }
    } catch (e) {
      console.error('Failed to load deployment detail:', e);
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  async function loadDeploymentYaml() {
    try {
      deploymentYaml = await invoke<string>('get_deployment_yaml', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      deploymentYaml = `Error loading YAML: ${e}`;
    }
  }

  async function loadDeploymentPods() {
    try {
      deploymentPods = await invoke<PodInfo[]>('get_deployment_pods', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      deploymentPods = [];
    }
  }

  async function loadDeploymentEvents() {
    try {
      deploymentEvents = await invoke<DeploymentEvent[]>('get_deployment_events', {
        contextName: context,
        namespace,
        name
      });
    } catch (e) {
      deploymentEvents = [];
    }
  }

  function handleTabChange(tab: TabType) {
    activeTab = tab;

    if (tab === 'yaml' && !deploymentYaml) {
      loadDeploymentYaml();
    } else if (tab === 'pods' && deploymentPods.length === 0) {
      loadDeploymentPods();
    } else if (tab === 'events' && deploymentEvents.length === 0) {
      loadDeploymentEvents();
    }
  }

  async function handleScale() {
    if (isDeleted || !deploymentDetail) return;

    try {
      isScaling = true;
      await invoke('scale_deployment', {
        namespace,
        name,
        replicas: scaleReplicas
      });
      showScaleModal = false;
      await loadDeploymentDetail();
      if (activeTab === 'pods') {
        await loadDeploymentPods();
      }
    } catch (e) {
      alert(`Failed to scale: ${e}`);
    } finally {
      isScaling = false;
    }
  }

  async function handleRestart() {
    if (isDeleted) return;

    if (!confirm(`Are you sure you want to restart deployment "${name}"?`)) {
      return;
    }

    try {
      isRestarting = true;
      await invoke('restart_deployment', {
        namespace,
        name
      });
      await loadDeploymentDetail();
      if (activeTab === 'pods') {
        await loadDeploymentPods();
      }
    } catch (e) {
      alert(`Failed to restart: ${e}`);
    } finally {
      isRestarting = false;
    }
  }

  async function openPodDetail(podName: string) {
    try {
      await invoke('open_resource_detail', {
        resourceType: 'pod',
        name: podName,
        namespace,
        context
      });
    } catch (e) {
      console.error('Failed to open pod detail:', e);
    }
  }

  function getStatusColor(status: string): string {
    switch (status.toLowerCase()) {
      case 'running': return 'text-accent-success bg-accent-success/10';
      case 'succeeded': return 'text-accent-success bg-accent-success/10';
      case 'pending': return 'text-accent-warning bg-accent-warning/10';
      case 'failed': return 'text-accent-error bg-accent-error/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
  }

  function getConditionColor(status: string): string {
    return status === 'True' ? 'text-accent-success' : 'text-accent-error';
  }

  function getEventTypeColor(type: string): string {
    return type === 'Normal' ? 'text-accent-success bg-accent-success/10' : 'text-accent-warning bg-accent-warning/10';
  }

  function isHealthy(): boolean {
    if (!deploymentDetail) return false;
    return deploymentDetail.ready_replicas === deploymentDetail.replicas && deploymentDetail.replicas > 0;
  }
</script>

<div class="h-full flex flex-col">
  <!-- Action bar -->
  <div class="flex items-center justify-between px-4 py-2 bg-bg-tertiary/50 border-b border-border-subtle">
    <div class="flex items-center gap-2">
      <span class="text-sm text-text-muted">Actions:</span>
    </div>
    <div class="flex items-center gap-2">
      <button
        onclick={() => { scaleReplicas = deploymentDetail?.replicas || 0; showScaleModal = true; }}
        disabled={isDeleted}
        class="px-3 py-1.5 text-sm bg-accent-primary/10 text-accent-primary rounded hover:bg-accent-primary/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
        </svg>
        Scale
      </button>
      <button
        onclick={handleRestart}
        disabled={isDeleted || isRestarting}
        class="px-3 py-1.5 text-sm bg-accent-warning/10 text-accent-warning rounded hover:bg-accent-warning/20 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1.5"
      >
        <svg class="w-4 h-4 {isRestarting ? 'animate-spin' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
        {isRestarting ? 'Restarting...' : 'Restart'}
      </button>
    </div>
  </div>

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
        onclick={() => handleTabChange('pods')}
        class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === 'pods' ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
      >
        Pods
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
          <span class="font-medium text-accent-warning">This deployment no longer exists</span>
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
          <p class="text-text-muted">Loading deployment details...</p>
        </div>
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-accent-error mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-muted">{error}</p>
          <button onclick={loadDeploymentDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">
            Retry
          </button>
        </div>
      </div>
    {:else if !deploymentDetail}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-text-muted mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M12 12h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-text-muted">No deployment data loaded</p>
          <button onclick={loadDeploymentDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">
            Retry
          </button>
        </div>
      </div>
    {:else if activeTab === 'overview'}
      <div class="p-6 space-y-6">
        <!-- Status Section -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Status</h2>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Health</div>
              <span class="text-sm px-2 py-0.5 rounded {isHealthy() ? 'text-accent-success bg-accent-success/10' : 'text-accent-warning bg-accent-warning/10'}">
                {isHealthy() ? 'Healthy' : 'Degraded'}
              </span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Ready</div>
              <span class="text-lg font-semibold text-text-primary">{deploymentDetail.ready_replicas}/{deploymentDetail.replicas}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Up-to-date</div>
              <span class="text-lg font-semibold text-text-primary">{deploymentDetail.updated_replicas}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Available</div>
              <span class="text-lg font-semibold text-text-primary">{deploymentDetail.available_replicas}</span>
            </div>
          </div>
        </section>

        <!-- Spec Section -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Spec</h2>
          <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Strategy</div>
              <span class="text-sm text-text-primary">{deploymentDetail.strategy}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Min Ready Seconds</div>
              <span class="text-sm text-text-primary">{deploymentDetail.min_ready_seconds}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Revision History</div>
              <span class="text-sm text-text-primary">{deploymentDetail.revision_history_limit ?? 'Unlimited'}</span>
            </div>
          </div>
        </section>

        <!-- Container Images -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">Container Images</h2>
          <div class="bg-bg-secondary rounded-lg p-4">
            {#if deploymentDetail.container_images.length > 0}
              <div class="space-y-2">
                {#each deploymentDetail.container_images as image}
                  <code class="block text-sm text-text-secondary break-all">{image}</code>
                {/each}
              </div>
            {:else}
              <p class="text-text-muted text-sm">No container images</p>
            {/if}
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
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Message</th>
                </tr>
              </thead>
              <tbody>
                {#each deploymentDetail.conditions as condition}
                  <tr class="border-t border-border-subtle">
                    <td class="px-4 py-2 text-text-primary">{condition.condition_type}</td>
                    <td class="px-4 py-2 {getConditionColor(condition.status)}">{condition.status}</td>
                    <td class="px-4 py-2 text-text-secondary">{condition.reason || '-'}</td>
                    <td class="px-4 py-2 text-text-muted text-xs">{condition.message || '-'}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </section>

        <!-- Labels & Selector -->
        <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Labels</h2>
            <div class="bg-bg-secondary rounded-lg p-4">
              {#if Object.keys(deploymentDetail.labels).length > 0}
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(deploymentDetail.labels) as [key, value]}
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
            <h2 class="text-lg font-semibold text-text-primary mb-4">Selector</h2>
            <div class="bg-bg-secondary rounded-lg p-4">
              {#if Object.keys(deploymentDetail.selector).length > 0}
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(deploymentDetail.selector) as [key, value]}
                    <span class="text-xs bg-accent-primary/10 text-accent-primary px-2 py-1 rounded">
                      {key}={value}
                    </span>
                  {/each}
                </div>
              {:else}
                <p class="text-text-muted text-sm">No selector</p>
              {/if}
            </div>
          </div>
        </section>
      </div>

    {:else if activeTab === 'pods'}
      <div class="p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-text-primary">Pods ({deploymentPods.length})</h2>
          <button
            onclick={loadDeploymentPods}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        {#if deploymentPods.length > 0}
          <div class="bg-bg-secondary rounded-lg overflow-hidden">
            <table class="w-full text-sm">
              <thead class="bg-bg-tertiary">
                <tr>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Name</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Status</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Ready</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Restarts</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Age</th>
                </tr>
              </thead>
              <tbody>
                {#each deploymentPods as pod}
                  <tr
                    class="border-t border-border-subtle hover:bg-bg-tertiary cursor-pointer"
                    onclick={() => openPodDetail(pod.name)}
                  >
                    <td class="px-4 py-2 text-accent-primary hover:underline">{pod.name}</td>
                    <td class="px-4 py-2">
                      <span class="text-xs px-2 py-0.5 rounded {getStatusColor(pod.status)}">{pod.status}</span>
                    </td>
                    <td class="px-4 py-2 text-text-secondary">{pod.ready}</td>
                    <td class="px-4 py-2 text-text-secondary">{pod.restarts}</td>
                    <td class="px-4 py-2 text-text-muted">{pod.age}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <div class="flex items-center justify-center h-48">
            <p class="text-text-muted">No pods found</p>
          </div>
        {/if}
      </div>

    {:else if activeTab === 'events'}
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <span class="text-sm text-text-muted">Deployment Events</span>
          <button
            onclick={loadDeploymentEvents}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        <div class="flex-1 overflow-auto p-4">
          {#if deploymentEvents.length > 0}
            <div class="space-y-3">
              {#each deploymentEvents as event}
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
      <div class="h-full flex flex-col">
        <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
          <span class="text-sm text-text-muted">Deployment YAML Manifest</span>
          <div class="flex items-center gap-2">
            <button
              onclick={() => navigator.clipboard.writeText(deploymentYaml)}
              class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors"
            >
              Copy
            </button>
            <button
              onclick={loadDeploymentYaml}
              disabled={isDeleted}
              class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
            >
              Refresh
            </button>
          </div>
        </div>
        <div class="flex-1 overflow-hidden">
          {#if deploymentYaml}
            <YamlEditor content={deploymentYaml} readonly={true} />
          {:else}
            <div class="flex items-center justify-center h-full text-text-muted">
              Loading YAML...
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Scale Modal -->
{#if showScaleModal}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-bg-secondary rounded-lg p-6 w-96 shadow-xl">
      <h3 class="text-lg font-semibold text-text-primary mb-4">Scale Deployment</h3>
      <p class="text-sm text-text-secondary mb-4">
        Set the number of replicas for <span class="text-accent-primary">{name}</span>
      </p>
      <div class="flex items-center gap-3 mb-6">
        <button
          onclick={() => scaleReplicas = Math.max(0, scaleReplicas - 1)}
          class="w-12 h-12 rounded-lg bg-bg-tertiary hover:bg-accent-error/20 hover:text-accent-error border border-border-subtle text-text-primary text-2xl font-bold transition-colors flex items-center justify-center"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
          </svg>
        </button>
        <input
          type="number"
          bind:value={scaleReplicas}
          min="0"
          class="w-24 text-center text-3xl font-bold bg-bg-primary border border-border-subtle rounded-lg py-3 text-text-primary focus:outline-none focus:border-accent-primary [appearance:textfield] [&::-webkit-outer-spin-button]:appearance-none [&::-webkit-inner-spin-button]:appearance-none"
        />
        <button
          onclick={() => scaleReplicas++}
          class="w-12 h-12 rounded-lg bg-bg-tertiary hover:bg-accent-success/20 hover:text-accent-success border border-border-subtle text-text-primary text-2xl font-bold transition-colors flex items-center justify-center"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
        </button>
      </div>
      <div class="flex justify-end gap-3">
        <button
          onclick={() => showScaleModal = false}
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleScale}
          disabled={isScaling}
          class="px-4 py-2 text-sm bg-accent-primary text-white rounded hover:bg-accent-primary/90 transition-colors disabled:opacity-50"
        >
          {isScaling ? 'Scaling...' : 'Scale'}
        </button>
      </div>
    </div>
  </div>
{/if}
