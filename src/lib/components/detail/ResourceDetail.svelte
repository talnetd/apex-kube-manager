<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import YamlEditorPanel from '../ui/YamlEditorPanel.svelte';
  import MetadataSection from '../ui/MetadataSection.svelte';
  import EventsTable from '../ui/EventsTable.svelte';
  import ConditionsTable from '../ui/ConditionsTable.svelte';

  interface Props {
    resourceType: string;
    context: string;
    namespace: string;
    name: string;
  }

  let { resourceType, context, namespace, name }: Props = $props();

  // Resource configuration - defines tabs and display names for each type
  const resourceConfig: Record<string, { displayName: string; tabs: string[]; clusterScoped?: boolean }> = {
    pod: { displayName: 'Pod', tabs: ['overview', 'events', 'yaml'] },
    deployment: { displayName: 'Deployment', tabs: ['overview', 'pods', 'events', 'yaml'] },
    statefulset: { displayName: 'StatefulSet', tabs: ['overview', 'pods', 'events', 'yaml'] },
    daemonset: { displayName: 'DaemonSet', tabs: ['overview', 'pods', 'events', 'yaml'] },
    replicaset: { displayName: 'ReplicaSet', tabs: ['overview', 'pods', 'events', 'yaml'] },
    job: { displayName: 'Job', tabs: ['overview', 'pods', 'events', 'yaml'] },
    cronjob: { displayName: 'CronJob', tabs: ['overview', 'events', 'yaml'] },
    service: { displayName: 'Service', tabs: ['overview', 'events', 'yaml'] },
    ingress: { displayName: 'Ingress', tabs: ['overview', 'events', 'yaml'] },
    configmap: { displayName: 'ConfigMap', tabs: ['overview', 'data', 'events', 'yaml'] },
    secret: { displayName: 'Secret', tabs: ['overview', 'data', 'events', 'yaml'] },
    networkpolicy: { displayName: 'NetworkPolicy', tabs: ['overview', 'ingress', 'egress', 'events', 'yaml'] },
    hpa: { displayName: 'HPA', tabs: ['overview', 'metrics', 'events', 'yaml'] },
    pv: { displayName: 'PersistentVolume', tabs: ['overview', 'events', 'yaml'], clusterScoped: true },
    pvc: { displayName: 'PersistentVolumeClaim', tabs: ['overview', 'events', 'yaml'] },
    namespace: { displayName: 'Namespace', tabs: ['overview', 'events', 'yaml'], clusterScoped: true },
    node: { displayName: 'Node', tabs: ['overview', 'pods', 'resources', 'events', 'yaml'], clusterScoped: true },
    serviceaccount: { displayName: 'ServiceAccount', tabs: ['overview', 'events', 'yaml'] },
  };

  const config = $derived(resourceConfig[resourceType] || { displayName: resourceType, tabs: ['overview', 'events', 'yaml'] });

  let activeTab = $state<string>('overview');

  // Generic state
  let detail = $state<Record<string, any> | null>(null);
  let yaml = $state<string>('');
  let events = $state<any[]>([]);
  let pods = $state<any[]>([]);
  let secretData = $state<Record<string, string>>({});

  let isLoading = $state<boolean>(true);
  let error = $state<string | null>(null);
  let isDeleted = $state<boolean>(false);
  let lastSeen = $state<string | null>(null);
  let existenceInterval: ReturnType<typeof setInterval> | null = null;

  // Expanded state for configmap/secret data keys
  let expandedKeys = $state<Set<string>>(new Set());
  // Revealed state for secret values
  let revealedKeys = $state<Set<string>>(new Set());

  onMount(async () => {
    await loadDetail();
    existenceInterval = setInterval(checkExists, 5000);
    return () => {
      if (existenceInterval) clearInterval(existenceInterval);
    };
  });

  async function checkExists() {
    if (isDeleted) return;
    try {
      await invoke(`get_${resourceType}_detail`, getInvokeParams());
      lastSeen = new Date().toLocaleTimeString();
    } catch (e) {
      const errorStr = String(e);
      if (errorStr.toLowerCase().includes('not found') || errorStr.toLowerCase().includes('404')) {
        isDeleted = true;
        if (existenceInterval) clearInterval(existenceInterval);
      }
    }
  }

  function getInvokeParams(): Record<string, any> {
    const params: Record<string, any> = { contextName: context, name };
    if (!config.clusterScoped) {
      params.namespace = namespace;
    }
    return params;
  }

  async function loadDetail() {
    try {
      isLoading = true;
      error = null;
      detail = await invoke(`get_${resourceType}_detail`, getInvokeParams());
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  async function loadYaml() {
    try {
      yaml = await invoke(`get_${resourceType}_yaml`, getInvokeParams());
    } catch (e) {
      yaml = `Error loading YAML: ${e}`;
    }
  }

  async function loadEvents() {
    try {
      events = await invoke(`get_${resourceType}_events`, getInvokeParams());
    } catch (e) {
      events = [];
    }
  }

  async function loadPods() {
    try {
      const params = getInvokeParams();
      // Different resources have different param names for getting pods
      if (resourceType === 'node') {
        pods = await invoke('get_node_pods', { contextName: context, nodeName: name });
      } else if (resourceType === 'job') {
        pods = await invoke('get_job_pods', { contextName: context, namespace, jobName: name });
      } else {
        // deployment, statefulset, daemonset, replicaset use the resource name pattern
        const cmdName = `get_${resourceType}_pods`;
        const paramName = `${resourceType}Name`;
        pods = await invoke(cmdName, { contextName: context, namespace, [paramName]: name });
      }
    } catch (e) {
      pods = [];
    }
  }

  async function loadSecretData() {
    try {
      secretData = await invoke('get_secret_data', { contextName: context, namespace, name });
    } catch (e) {
      secretData = {};
    }
  }

  function handleTabChange(tab: string) {
    activeTab = tab;
    if (tab === 'yaml' && !yaml) loadYaml();
    else if (tab === 'events' && events.length === 0) loadEvents();
    else if (tab === 'pods' && pods.length === 0) loadPods();
    else if (tab === 'data' && resourceType === 'secret' && Object.keys(secretData).length === 0) loadSecretData();
  }

  function toggleKey(key: string) {
    if (expandedKeys.has(key)) {
      expandedKeys.delete(key);
      expandedKeys = new Set(expandedKeys);
    } else {
      expandedKeys.add(key);
      expandedKeys = new Set(expandedKeys);
    }
  }

  function toggleReveal(key: string) {
    if (revealedKeys.has(key)) {
      revealedKeys.delete(key);
      revealedKeys = new Set(revealedKeys);
    } else {
      revealedKeys.add(key);
      revealedKeys = new Set(revealedKeys);
    }
  }

  // Status helpers
  function getPodStatusColor(status: string): string {
    switch (status) {
      case 'Running': return 'text-accent-success bg-accent-success/10';
      case 'Succeeded': return 'text-accent-primary bg-accent-primary/10';
      case 'Pending': return 'text-accent-warning bg-accent-warning/10';
      case 'Failed': return 'text-accent-error bg-accent-error/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
  }

  function getConditionColor(status: string): string {
    return status === 'True' ? 'text-accent-success' : status === 'False' ? 'text-accent-error' : 'text-text-muted';
  }

  // Tab label display
  function getTabLabel(tab: string): string {
    const labels: Record<string, string> = {
      overview: 'Overview',
      pods: 'Pods',
      events: 'Events',
      yaml: 'YAML',
      data: 'Data',
      ingress: 'Ingress Rules',
      egress: 'Egress Rules',
      metrics: 'Metrics',
      resources: 'Resources',
    };
    return labels[tab] || tab;
  }
</script>

<div class="h-full flex flex-col">
  <!-- Tab Bar -->
  <div class="border-b border-border-subtle bg-bg-secondary">
    <nav class="flex gap-1 px-4">
      {#each config.tabs as tab}
        <button
          onclick={() => handleTabChange(tab)}
          class="px-4 py-3 text-sm font-medium border-b-2 transition-colors {activeTab === tab ? 'border-accent-primary text-accent-primary' : 'border-transparent text-text-muted hover:text-text-primary'}"
        >
          {getTabLabel(tab)}
        </button>
      {/each}
    </nav>
  </div>

  <!-- Deleted Banner -->
  {#if isDeleted}
    <div class="flex items-center justify-between px-4 py-3 bg-accent-warning/10 border-b border-accent-warning/30">
      <div class="flex items-center gap-3">
        <svg class="w-5 h-5 text-accent-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div>
          <span class="font-medium text-accent-warning">This {config.displayName} no longer exists</span>
          {#if lastSeen}
            <span class="text-text-muted text-sm ml-2">Last seen: {lastSeen}</span>
          {/if}
        </div>
      </div>
      <span class="text-xs text-text-muted bg-bg-tertiary px-2 py-1 rounded">Read-only</span>
    </div>
  {/if}

  <!-- Main Content -->
  <div class="flex-1 overflow-auto">
    {#if isLoading}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-8 h-8 text-accent-primary mx-auto mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <p class="text-text-muted">Loading {config.displayName} details...</p>
        </div>
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-accent-error mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-muted">{error}</p>
          <button onclick={loadDetail} class="mt-4 px-4 py-2 bg-accent-primary text-white rounded-lg hover:bg-accent-primary/90">Retry</button>
        </div>
      </div>
    {:else if !detail}
      <div class="flex items-center justify-center h-full">
        <p class="text-text-muted">No {config.displayName} data loaded</p>
      </div>

    <!-- OVERVIEW TAB -->
    {:else if activeTab === 'overview'}
      <div class="p-6 space-y-6">
        <!-- Basic Info - Common to all -->
        <section>
          <h2 class="text-lg font-semibold text-text-primary mb-4">{config.displayName} Info</h2>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Name</div>
              <span class="text-sm text-text-primary font-mono truncate block">{detail.name}</span>
            </div>
            {#if detail.namespace}
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Namespace</div>
                <span class="text-sm text-text-primary">{detail.namespace}</span>
              </div>
            {/if}
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">UID</div>
              <span class="text-xs text-text-secondary font-mono truncate block">{detail.uid}</span>
            </div>
            <div class="bg-bg-secondary rounded-lg p-4">
              <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Created</div>
              <span class="text-sm text-text-primary">{detail.creation_timestamp}</span>
            </div>
          </div>
        </section>

        <!-- Resource-specific Overview Sections -->

        <!-- DEPLOYMENT -->
        {#if resourceType === 'deployment'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Replica Status</h2>
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Desired</div>
                <span class="text-xl font-bold text-text-primary">{detail.replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Ready</div>
                <span class="text-xl font-bold text-accent-success">{detail.ready_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Available</div>
                <span class="text-xl font-bold text-accent-primary">{detail.available_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Up-to-date</div>
                <span class="text-xl font-bold text-text-primary">{detail.updated_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Unavailable</div>
                <span class="text-xl font-bold {(detail.unavailable_replicas ?? 0) > 0 ? 'text-accent-error' : 'text-text-primary'}">{detail.unavailable_replicas ?? 0}</span>
              </div>
            </div>
          </section>
          {#if detail.strategy}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Strategy</h2>
              <div class="bg-bg-secondary rounded-lg p-4">
                <span class="text-sm px-2 py-0.5 rounded bg-accent-primary/10 text-accent-primary">{detail.strategy}</span>
              </div>
            </section>
          {/if}

        <!-- STATEFULSET -->
        {:else if resourceType === 'statefulset'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Replica Status</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Desired</div>
                <span class="text-xl font-bold text-text-primary">{detail.replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Ready</div>
                <span class="text-xl font-bold text-accent-success">{detail.ready_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Current</div>
                <span class="text-xl font-bold text-accent-primary">{detail.current_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Updated</div>
                <span class="text-xl font-bold text-text-primary">{detail.updated_replicas ?? 0}</span>
              </div>
            </div>
          </section>
          {#if detail.service_name}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Service</h2>
              <div class="bg-bg-secondary rounded-lg p-4">
                <span class="text-sm text-accent-primary font-mono">{detail.service_name}</span>
              </div>
            </section>
          {/if}

        <!-- DAEMONSET -->
        {:else if resourceType === 'daemonset'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Scheduling Status</h2>
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Desired</div>
                <span class="text-xl font-bold text-text-primary">{detail.desired_number_scheduled ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Current</div>
                <span class="text-xl font-bold text-accent-primary">{detail.current_number_scheduled ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Ready</div>
                <span class="text-xl font-bold text-accent-success">{detail.number_ready ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Available</div>
                <span class="text-xl font-bold text-accent-success">{detail.number_available ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Misscheduled</div>
                <span class="text-xl font-bold {(detail.number_misscheduled ?? 0) > 0 ? 'text-accent-error' : 'text-text-primary'}">{detail.number_misscheduled ?? 0}</span>
              </div>
            </div>
          </section>

        <!-- REPLICASET -->
        {:else if resourceType === 'replicaset'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Replica Status</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Desired</div>
                <span class="text-xl font-bold text-text-primary">{detail.replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Ready</div>
                <span class="text-xl font-bold text-accent-success">{detail.ready_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Available</div>
                <span class="text-xl font-bold text-accent-primary">{detail.available_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Fully Labeled</div>
                <span class="text-xl font-bold text-text-primary">{detail.fully_labeled_replicas ?? 0}</span>
              </div>
            </div>
          </section>
          {#if detail.owner_references && detail.owner_references.length > 0}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Owner</h2>
              <div class="bg-bg-secondary rounded-lg p-4">
                {#each detail.owner_references as owner}
                  <div class="flex items-center gap-2">
                    <span class="text-xs px-2 py-0.5 rounded bg-accent-primary/10 text-accent-primary">{owner.kind}</span>
                    <span class="text-sm text-text-primary font-mono">{owner.name}</span>
                  </div>
                {/each}
              </div>
            </section>
          {/if}

        <!-- JOB -->
        {:else if resourceType === 'job'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Job Status</h2>
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Status</div>
                <span class="text-sm px-2 py-0.5 rounded {detail.completion_time ? 'text-accent-success bg-accent-success/10' : detail.active > 0 ? 'text-accent-primary bg-accent-primary/10' : 'text-accent-warning bg-accent-warning/10'}">
                  {detail.completion_time ? 'Complete' : detail.active > 0 ? 'Running' : 'Pending'}
                </span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Active</div>
                <span class="text-xl font-bold text-accent-primary">{detail.active ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Succeeded</div>
                <span class="text-xl font-bold text-accent-success">{detail.succeeded ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Failed</div>
                <span class="text-xl font-bold {(detail.failed ?? 0) > 0 ? 'text-accent-error' : 'text-text-primary'}">{detail.failed ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Completions</div>
                <span class="text-sm text-text-primary">{detail.succeeded ?? 0}/{detail.completions ?? '∞'}</span>
              </div>
            </div>
          </section>
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Configuration</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Parallelism</div>
                <span class="text-sm text-text-primary">{detail.parallelism ?? 1}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Backoff Limit</div>
                <span class="text-sm text-text-primary">{detail.backoff_limit ?? 6}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Completion Mode</div>
                <span class="text-sm text-text-primary">{detail.completion_mode || 'NonIndexed'}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">TTL After Finished</div>
                <span class="text-sm text-text-primary">{detail.ttl_seconds_after_finished ? `${detail.ttl_seconds_after_finished}s` : 'Never'}</span>
              </div>
            </div>
          </section>

        <!-- CRONJOB -->
        {:else if resourceType === 'cronjob'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Schedule</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Schedule</div>
                <code class="text-sm text-accent-primary">{detail.schedule}</code>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Suspended</div>
                <span class="text-sm px-2 py-0.5 rounded {detail.suspend ? 'text-accent-warning bg-accent-warning/10' : 'text-accent-success bg-accent-success/10'}">
                  {detail.suspend ? 'Yes' : 'No'}
                </span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Active Jobs</div>
                <span class="text-xl font-bold text-accent-primary">{detail.active_jobs ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Last Schedule</div>
                <span class="text-sm text-text-primary">{detail.last_schedule_time || 'Never'}</span>
              </div>
            </div>
          </section>
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Configuration</h2>
            <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Concurrency Policy</div>
                <span class="text-sm text-text-primary">{detail.concurrency_policy || 'Allow'}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Success History</div>
                <span class="text-sm text-text-primary">{detail.successful_jobs_history_limit ?? 3}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Failed History</div>
                <span class="text-sm text-text-primary">{detail.failed_jobs_history_limit ?? 1}</span>
              </div>
            </div>
          </section>

        <!-- SERVICE -->
        {:else if resourceType === 'service'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Service Info</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Type</div>
                <span class="text-sm px-2 py-0.5 rounded bg-accent-primary/10 text-accent-primary">{detail.service_type}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Cluster IP</div>
                <span class="text-sm text-text-primary font-mono">{detail.cluster_ip || 'None'}</span>
              </div>
              {#if detail.external_ips && detail.external_ips.length > 0}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">External IPs</div>
                  <span class="text-sm text-text-primary font-mono">{detail.external_ips.join(', ')}</span>
                </div>
              {/if}
              {#if detail.load_balancer_ip}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Load Balancer IP</div>
                  <span class="text-sm text-text-primary font-mono">{detail.load_balancer_ip}</span>
                </div>
              {/if}
            </div>
          </section>
          {#if detail.ports && detail.ports.length > 0}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Ports</h2>
              <div class="bg-bg-secondary rounded-lg overflow-hidden">
                <table class="w-full text-sm">
                  <thead class="bg-bg-tertiary">
                    <tr>
                      <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Name</th>
                      <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Protocol</th>
                      <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Port</th>
                      <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Target Port</th>
                      <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Node Port</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each detail.ports as port}
                      <tr class="border-t border-border-subtle">
                        <td class="px-4 py-2 text-text-primary">{port.name || '-'}</td>
                        <td class="px-4 py-2 text-text-secondary">{port.protocol}</td>
                        <td class="px-4 py-2 text-accent-primary font-mono">{port.port}</td>
                        <td class="px-4 py-2 text-text-secondary font-mono">{port.target_port}</td>
                        <td class="px-4 py-2 text-text-secondary font-mono">{port.node_port || '-'}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </section>
          {/if}

        <!-- INGRESS -->
        {:else if resourceType === 'ingress'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Ingress Info</h2>
            <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
              {#if detail.ingress_class_name}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Ingress Class</div>
                  <span class="text-sm text-accent-primary">{detail.ingress_class_name}</span>
                </div>
              {/if}
              {#if detail.load_balancer_ips && detail.load_balancer_ips.length > 0}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Load Balancer</div>
                  <span class="text-sm text-text-primary font-mono">{detail.load_balancer_ips.join(', ')}</span>
                </div>
              {/if}
            </div>
          </section>
          {#if detail.rules && detail.rules.length > 0}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Rules</h2>
              <div class="space-y-3">
                {#each detail.rules as rule}
                  <div class="bg-bg-secondary rounded-lg p-4">
                    <div class="text-sm font-medium text-accent-primary mb-2">{rule.host || '*'}</div>
                    {#if rule.paths && rule.paths.length > 0}
                      <div class="space-y-1">
                        {#each rule.paths as path}
                          <div class="flex items-center gap-2 text-sm">
                            <code class="text-text-secondary">{path.path || '/'}</code>
                            <span class="text-text-muted">→</span>
                            <span class="text-text-primary">{path.backend_service}:{path.backend_port}</span>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>
                {/each}
              </div>
            </section>
          {/if}

        <!-- CONFIGMAP -->
        {:else if resourceType === 'configmap'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Data Summary</h2>
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Data Keys</div>
                <span class="text-xl font-bold text-accent-primary">{Object.keys(detail.data || {}).length}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Binary Keys</div>
                <span class="text-xl font-bold text-text-primary">{(detail.binary_data_keys || []).length}</span>
              </div>
            </div>
          </section>

        <!-- SECRET -->
        {:else if resourceType === 'secret'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Secret Info</h2>
            <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Type</div>
                <span class="text-sm text-accent-warning">{detail.secret_type}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Data Keys</div>
                <span class="text-xl font-bold text-accent-primary">{(detail.data_keys || []).length}</span>
              </div>
            </div>
          </section>

        <!-- NETWORKPOLICY -->
        {:else if resourceType === 'networkpolicy'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Policy Info</h2>
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Pod Selector</div>
                <code class="text-sm text-text-primary">{detail.pod_selector || '<all pods>'}</code>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Policy Types</div>
                <div class="flex gap-2">
                  {#each (detail.policy_types || []) as ptype}
                    <span class="text-xs px-2 py-0.5 rounded {ptype === 'Ingress' ? 'bg-accent-success/10 text-accent-success' : 'bg-accent-warning/10 text-accent-warning'}">{ptype}</span>
                  {/each}
                </div>
              </div>
            </div>
          </section>

        <!-- HPA -->
        {:else if resourceType === 'hpa'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Autoscaling Status</h2>
            <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Target</div>
                <span class="text-sm text-accent-primary">{detail.scale_target_ref}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Min Replicas</div>
                <span class="text-xl font-bold text-text-primary">{detail.min_replicas ?? 1}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Max Replicas</div>
                <span class="text-xl font-bold text-text-primary">{detail.max_replicas}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Current</div>
                <span class="text-xl font-bold text-accent-success">{detail.current_replicas ?? 0}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Desired</div>
                <span class="text-xl font-bold text-accent-primary">{detail.desired_replicas ?? 0}</span>
              </div>
            </div>
          </section>

        <!-- PV -->
        {:else if resourceType === 'pv'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Volume Info</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Status</div>
                <span class="text-sm px-2 py-0.5 rounded {detail.phase === 'Bound' ? 'text-accent-success bg-accent-success/10' : detail.phase === 'Available' ? 'text-accent-primary bg-accent-primary/10' : 'text-accent-warning bg-accent-warning/10'}">{detail.phase}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Capacity</div>
                <span class="text-xl font-bold text-accent-primary">{detail.capacity}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Access Modes</div>
                <span class="text-sm text-text-primary">{(detail.access_modes || []).join(', ')}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Reclaim Policy</div>
                <span class="text-sm text-text-primary">{detail.reclaim_policy}</span>
              </div>
            </div>
          </section>
          {#if detail.claim_ref}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Bound Claim</h2>
              <div class="bg-bg-secondary rounded-lg p-4">
                <span class="text-sm text-accent-primary font-mono">{detail.claim_ref}</span>
              </div>
            </section>
          {/if}

        <!-- PVC -->
        {:else if resourceType === 'pvc'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Claim Info</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Status</div>
                <span class="text-sm px-2 py-0.5 rounded {detail.phase === 'Bound' ? 'text-accent-success bg-accent-success/10' : 'text-accent-warning bg-accent-warning/10'}">{detail.phase}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Capacity</div>
                <span class="text-xl font-bold text-accent-primary">{detail.capacity || 'Pending'}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Access Modes</div>
                <span class="text-sm text-text-primary">{(detail.access_modes || []).join(', ')}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Storage Class</div>
                <span class="text-sm text-text-primary">{detail.storage_class || 'Default'}</span>
              </div>
            </div>
          </section>
          {#if detail.volume_name}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Bound Volume</h2>
              <div class="bg-bg-secondary rounded-lg p-4">
                <span class="text-sm text-accent-primary font-mono">{detail.volume_name}</span>
              </div>
            </section>
          {/if}

        <!-- NAMESPACE -->
        {:else if resourceType === 'namespace'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Namespace Status</h2>
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Status</div>
                <span class="text-sm px-2 py-0.5 rounded {detail.phase === 'Active' ? 'text-accent-success bg-accent-success/10' : 'text-accent-warning bg-accent-warning/10'}">{detail.phase}</span>
              </div>
              {#if detail.finalizers && detail.finalizers.length > 0}
                <div class="bg-bg-secondary rounded-lg p-4">
                  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Finalizers</div>
                  <div class="flex flex-wrap gap-1">
                    {#each detail.finalizers as f}
                      <span class="text-xs bg-bg-tertiary px-2 py-0.5 rounded text-text-secondary">{f}</span>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </section>

        <!-- NODE -->
        {:else if resourceType === 'node'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">Node Info</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Status</div>
                <span class="text-sm px-2 py-0.5 rounded {detail.status === 'Ready' ? 'text-accent-success bg-accent-success/10' : 'text-accent-error bg-accent-error/10'}">{detail.status}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Roles</div>
                <span class="text-sm text-text-primary">{(detail.roles || []).join(', ') || 'worker'}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Kubelet Version</div>
                <span class="text-sm text-text-primary">{detail.kubelet_version}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Internal IP</div>
                <span class="text-sm text-text-primary font-mono">{detail.internal_ip || 'N/A'}</span>
              </div>
            </div>
          </section>
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">System Info</h2>
            <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">OS</div>
                <span class="text-sm text-text-primary">{detail.os_image}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Kernel</div>
                <span class="text-sm text-text-primary">{detail.kernel_version}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Container Runtime</div>
                <span class="text-sm text-text-primary">{detail.container_runtime_version}</span>
              </div>
            </div>
          </section>

        <!-- SERVICEACCOUNT -->
        {:else if resourceType === 'serviceaccount'}
          <section>
            <h2 class="text-lg font-semibold text-text-primary mb-4">ServiceAccount Info</h2>
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Secrets</div>
                <span class="text-xl font-bold text-accent-primary">{(detail.secrets || []).length}</span>
              </div>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-xs text-text-muted uppercase tracking-wide mb-1">Image Pull Secrets</div>
                <span class="text-xl font-bold text-text-primary">{(detail.image_pull_secrets || []).length}</span>
              </div>
            </div>
          </section>
          {#if detail.secrets && detail.secrets.length > 0}
            <section>
              <h2 class="text-lg font-semibold text-text-primary mb-4">Secrets</h2>
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="space-y-2">
                  {#each detail.secrets as secret}
                    <span class="block text-sm text-accent-primary font-mono">{secret}</span>
                  {/each}
                </div>
              </div>
            </section>
          {/if}
        {/if}

        <!-- Conditions - Common for resources that have them -->
        {#if detail.conditions && detail.conditions.length > 0}
          <ConditionsTable conditions={detail.conditions} />
        {/if}

        <!-- Metadata - Common to all -->
        <MetadataSection labels={detail.labels || {}} annotations={detail.annotations || {}} />
      </div>

    <!-- PODS TAB -->
    {:else if activeTab === 'pods'}
      <div class="p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-text-primary">Pods ({pods.length})</h2>
          <button
            onclick={loadPods}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        {#if pods.length > 0}
          <div class="bg-bg-secondary rounded-lg overflow-hidden">
            <table class="w-full text-sm">
              <thead class="bg-bg-tertiary">
                <tr>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Name</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Status</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Ready</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Restarts</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Age</th>
                  {#if resourceType === 'node'}
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Namespace</th>
                  {/if}
                </tr>
              </thead>
              <tbody>
                {#each pods as pod}
                  <tr class="border-t border-border-subtle hover:bg-bg-tertiary">
                    <td class="px-4 py-2 text-text-primary font-mono text-xs">{pod.name}</td>
                    <td class="px-4 py-2">
                      <span class="text-xs px-2 py-0.5 rounded {getPodStatusColor(pod.status)}">{pod.status}</span>
                    </td>
                    <td class="px-4 py-2 text-text-secondary">{pod.ready}</td>
                    <td class="px-4 py-2 text-text-secondary">{pod.restarts}</td>
                    <td class="px-4 py-2 text-text-muted">{pod.age}</td>
                    {#if resourceType === 'node'}
                      <td class="px-4 py-2 text-text-secondary">{pod.namespace}</td>
                    {/if}
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

    <!-- DATA TAB (ConfigMap/Secret) -->
    {:else if activeTab === 'data'}
      <div class="p-6">
        {#if resourceType === 'configmap'}
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-text-primary">Data ({Object.keys(detail.data || {}).length} keys)</h2>
          </div>
          {#if Object.keys(detail.data || {}).length > 0}
            <div class="space-y-3">
              {#each Object.entries(detail.data || {}) as [key, value]}
                <div class="bg-bg-secondary rounded-lg overflow-hidden">
                  <button
                    onclick={() => toggleKey(key)}
                    class="w-full flex items-center justify-between px-4 py-3 hover:bg-bg-tertiary transition-colors"
                  >
                    <span class="font-mono text-sm text-accent-primary">{key}</span>
                    <div class="flex items-center gap-2">
                      <span class="text-xs text-text-muted">{String(value).length} chars</span>
                      <svg class="w-4 h-4 text-text-muted transition-transform {expandedKeys.has(key) ? 'rotate-180' : ''}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                      </svg>
                    </div>
                  </button>
                  {#if expandedKeys.has(key)}
                    <div class="border-t border-border-subtle">
                      <pre class="p-4 text-xs text-text-secondary font-mono whitespace-pre-wrap break-all overflow-auto max-h-96">{value}</pre>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {:else}
            <div class="flex items-center justify-center h-48">
              <p class="text-text-muted">No data keys</p>
            </div>
          {/if}
        {:else if resourceType === 'secret'}
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-text-primary">Secret Data ({(detail.data_keys || []).length} keys)</h2>
            <button
              onclick={loadSecretData}
              disabled={isDeleted}
              class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
            >
              {Object.keys(secretData).length > 0 ? 'Refresh' : 'Load Data'}
            </button>
          </div>
          {#if (detail.data_keys || []).length > 0}
            <div class="space-y-3">
              {#each (detail.data_keys || []) as key}
                <div class="bg-bg-secondary rounded-lg overflow-hidden">
                  <div class="flex items-center justify-between px-4 py-3">
                    <span class="font-mono text-sm text-accent-warning">{key}</span>
                    <div class="flex items-center gap-2">
                      {#if secretData[key]}
                        <button
                          onclick={() => toggleReveal(key)}
                          class="text-xs px-2 py-1 rounded bg-bg-tertiary hover:bg-accent-warning/20 text-text-muted hover:text-accent-warning transition-colors"
                        >
                          {revealedKeys.has(key) ? 'Hide' : 'Reveal'}
                        </button>
                      {:else}
                        <span class="text-xs text-text-muted">Not loaded</span>
                      {/if}
                    </div>
                  </div>
                  {#if revealedKeys.has(key) && secretData[key]}
                    <div class="border-t border-border-subtle">
                      <pre class="p-4 text-xs text-text-secondary font-mono whitespace-pre-wrap break-all overflow-auto max-h-96">{secretData[key]}</pre>
                    </div>
                  {/if}
                </div>
              {/each}
            </div>
          {:else}
            <div class="flex items-center justify-center h-48">
              <p class="text-text-muted">No data keys</p>
            </div>
          {/if}
        {/if}
      </div>

    <!-- INGRESS RULES TAB (NetworkPolicy) -->
    {:else if activeTab === 'ingress' && resourceType === 'networkpolicy'}
      <div class="p-6">
        <h2 class="text-lg font-semibold text-text-primary mb-4">Ingress Rules</h2>
        {#if detail.ingress_rules && detail.ingress_rules.length > 0}
          <div class="space-y-4">
            {#each detail.ingress_rules as rule, i}
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-sm font-medium text-accent-success mb-3">Rule {i + 1}</div>
                {#if rule.from && rule.from.length > 0}
                  <div class="mb-3">
                    <div class="text-xs text-text-muted uppercase mb-2">From</div>
                    <div class="space-y-2">
                      {#each rule.from as peer}
                        <div class="text-sm bg-bg-tertiary rounded px-3 py-2">
                          {#if peer.pod_selector}
                            <span class="text-accent-primary">pods: </span><code class="text-text-secondary">{peer.pod_selector}</code>
                          {/if}
                          {#if peer.namespace_selector}
                            <span class="text-accent-warning">ns: </span><code class="text-text-secondary">{peer.namespace_selector}</code>
                          {/if}
                          {#if peer.ip_block}
                            <span class="text-accent-error">cidr: </span><code class="text-text-secondary">{peer.ip_block}</code>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                {:else}
                  <div class="text-sm text-text-muted mb-3">From: All sources</div>
                {/if}
                {#if rule.ports && rule.ports.length > 0}
                  <div>
                    <div class="text-xs text-text-muted uppercase mb-2">Ports</div>
                    <div class="flex flex-wrap gap-2">
                      {#each rule.ports as port}
                        <span class="text-xs bg-bg-tertiary px-2 py-1 rounded text-text-primary">{port.protocol}/{port.port}</span>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {:else}
          <div class="flex items-center justify-center h-48">
            <p class="text-text-muted">No ingress rules defined</p>
          </div>
        {/if}
      </div>

    <!-- EGRESS RULES TAB (NetworkPolicy) -->
    {:else if activeTab === 'egress' && resourceType === 'networkpolicy'}
      <div class="p-6">
        <h2 class="text-lg font-semibold text-text-primary mb-4">Egress Rules</h2>
        {#if detail.egress_rules && detail.egress_rules.length > 0}
          <div class="space-y-4">
            {#each detail.egress_rules as rule, i}
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="text-sm font-medium text-accent-warning mb-3">Rule {i + 1}</div>
                {#if rule.to && rule.to.length > 0}
                  <div class="mb-3">
                    <div class="text-xs text-text-muted uppercase mb-2">To</div>
                    <div class="space-y-2">
                      {#each rule.to as peer}
                        <div class="text-sm bg-bg-tertiary rounded px-3 py-2">
                          {#if peer.pod_selector}
                            <span class="text-accent-primary">pods: </span><code class="text-text-secondary">{peer.pod_selector}</code>
                          {/if}
                          {#if peer.namespace_selector}
                            <span class="text-accent-warning">ns: </span><code class="text-text-secondary">{peer.namespace_selector}</code>
                          {/if}
                          {#if peer.ip_block}
                            <span class="text-accent-error">cidr: </span><code class="text-text-secondary">{peer.ip_block}</code>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                {:else}
                  <div class="text-sm text-text-muted mb-3">To: All destinations</div>
                {/if}
                {#if rule.ports && rule.ports.length > 0}
                  <div>
                    <div class="text-xs text-text-muted uppercase mb-2">Ports</div>
                    <div class="flex flex-wrap gap-2">
                      {#each rule.ports as port}
                        <span class="text-xs bg-bg-tertiary px-2 py-1 rounded text-text-primary">{port.protocol}/{port.port}</span>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {:else}
          <div class="flex items-center justify-center h-48">
            <p class="text-text-muted">No egress rules defined</p>
          </div>
        {/if}
      </div>

    <!-- METRICS TAB (HPA) -->
    {:else if activeTab === 'metrics' && resourceType === 'hpa'}
      <div class="p-6">
        <h2 class="text-lg font-semibold text-text-primary mb-4">Metrics</h2>
        {#if detail.metrics && detail.metrics.length > 0}
          <div class="space-y-4">
            {#each detail.metrics as metric}
              <div class="bg-bg-secondary rounded-lg p-4">
                <div class="flex items-center justify-between mb-2">
                  <span class="text-sm font-medium text-text-primary">{metric.name || metric.metric_type}</span>
                  <span class="text-xs px-2 py-0.5 rounded bg-accent-primary/10 text-accent-primary">{metric.metric_type}</span>
                </div>
                <div class="grid grid-cols-2 gap-4 mt-3">
                  <div>
                    <div class="text-xs text-text-muted">Current</div>
                    <span class="text-lg font-bold text-accent-success">{metric.current_value || 'N/A'}</span>
                  </div>
                  <div>
                    <div class="text-xs text-text-muted">Target</div>
                    <span class="text-lg font-bold text-text-primary">{metric.target_value || 'N/A'}</span>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="flex items-center justify-center h-48">
            <p class="text-text-muted">No metrics configured</p>
          </div>
        {/if}
      </div>

    <!-- RESOURCES TAB (Node) -->
    {:else if activeTab === 'resources' && resourceType === 'node'}
      <div class="p-6">
        <h2 class="text-lg font-semibold text-text-primary mb-4">Resource Capacity</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="bg-bg-secondary rounded-lg p-4">
            <h3 class="text-sm font-medium text-text-primary mb-4">Capacity</h3>
            <div class="space-y-3">
              {#each Object.entries(detail.capacity || {}) as [key, value]}
                <div class="flex justify-between items-center">
                  <span class="text-sm text-text-muted">{key}</span>
                  <span class="text-sm text-text-primary font-mono">{value}</span>
                </div>
              {/each}
            </div>
          </div>
          <div class="bg-bg-secondary rounded-lg p-4">
            <h3 class="text-sm font-medium text-text-primary mb-4">Allocatable</h3>
            <div class="space-y-3">
              {#each Object.entries(detail.allocatable || {}) as [key, value]}
                <div class="flex justify-between items-center">
                  <span class="text-sm text-text-muted">{key}</span>
                  <span class="text-sm text-accent-success font-mono">{value}</span>
                </div>
              {/each}
            </div>
          </div>
        </div>
        {#if detail.taints && detail.taints.length > 0}
          <h2 class="text-lg font-semibold text-text-primary mt-6 mb-4">Taints</h2>
          <div class="bg-bg-secondary rounded-lg overflow-hidden">
            <table class="w-full text-sm">
              <thead class="bg-bg-tertiary">
                <tr>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Key</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Value</th>
                  <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Effect</th>
                </tr>
              </thead>
              <tbody>
                {#each detail.taints as taint}
                  <tr class="border-t border-border-subtle">
                    <td class="px-4 py-2 text-text-primary font-mono">{taint.key}</td>
                    <td class="px-4 py-2 text-text-secondary">{taint.value || '-'}</td>
                    <td class="px-4 py-2">
                      <span class="text-xs px-2 py-0.5 rounded {taint.effect === 'NoSchedule' ? 'bg-accent-error/10 text-accent-error' : taint.effect === 'PreferNoSchedule' ? 'bg-accent-warning/10 text-accent-warning' : 'bg-accent-primary/10 text-accent-primary'}">{taint.effect}</span>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>

    <!-- EVENTS TAB -->
    {:else if activeTab === 'events'}
      <EventsTable
        events={events}
        isLoading={false}
        isDeleted={isDeleted}
        onRefresh={loadEvents}
        resourceType={config.displayName}
      />

    <!-- YAML TAB -->
    {:else if activeTab === 'yaml'}
      <YamlEditorPanel
        yaml={yaml}
        context={context}
        resourceType={config.displayName}
        isDeleted={isDeleted}
        onRefresh={loadYaml}
        onApplySuccess={loadDetail}
      />
    {/if}
  </div>
</div>
