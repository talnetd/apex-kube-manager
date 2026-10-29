<script lang="ts">
  import { guard, detailReadOnly } from '../../stores/detail';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import YamlEditorPanel from '../ui/YamlEditorPanel.svelte';
  import EventsTable from '../ui/EventsTable.svelte';
  import ConditionsTable from '../ui/ConditionsTable.svelte';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import Button from '../ui/Button.svelte';
  import DetailToolbar from './kit/DetailToolbar.svelte';
  import DetailTabs from './kit/DetailTabs.svelte';
  import type { Verdict } from '../../utils/health';

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
    verdict?: Verdict;
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
  
  let scaleReplicas = $state<number>(0);

  // Restart state
  let isRestarting = $state<boolean>(false);

  // Tombstone state
  let isDeleted = $state<boolean>(false);
  let lastSeen = $state<string | null>(null);
  let existenceInterval: ReturnType<typeof setInterval> | null = null;

  // Sync onMount so the cleanup runs (an async one's return is ignored).
  onMount(() => {
    loadDeploymentDetail();
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

  // Scale and restart go through ConfirmDialog (prod guard, typed confirm on
  // production). Errors surface in the dialog; the read-only guard runs at
  // open and again on confirm, since the app can switch context in between.
  let confirmAction = $state<'scale' | 'restart' | null>(null);
  const replicasValid = $derived(Number.isInteger(scaleReplicas) && scaleReplicas >= 0);

  function askScale() {
    if (!guard('scale') || isDeleted || !deploymentDetail) return;
    scaleReplicas = deploymentDetail.replicas || 0;
    confirmAction = 'scale';
  }

  function askRestart() {
    if (!guard('restart') || isDeleted) return;
    confirmAction = 'restart';
  }

  async function doScale() {
    if (!guard('scale')) throw new Error('Switch back to this context to scale');
    if (!replicasValid) throw new Error('Replicas must be a whole number, 0 or more');
    await invoke('scale_deployment', {
      // The cluster this window shows, not kubeconfig's current one.
      contextName: context,
      namespace,
      name,
      replicas: scaleReplicas,
    });
    await loadDeploymentDetail();
    if (activeTab === 'pods') await loadDeploymentPods();
  }

  async function doRestart() {
    if (!guard('restart')) throw new Error('Switch back to this context to restart');
    isRestarting = true;
    try {
      await invoke('restart_deployment', { contextName: context, namespace, name });
      await loadDeploymentDetail();
      if (activeTab === 'pods') await loadDeploymentPods();
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
      case 'running': return 'text-success bg-accent-success/10';
      case 'succeeded': return 'text-success bg-accent-success/10';
      case 'pending': return 'text-warning bg-accent-warning/10';
      case 'failed': return 'text-danger bg-accent-error/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
  }



</script>

<div class="h-full flex flex-col">
  <DetailToolbar
    verdict={deploymentDetail?.verdict}
    summary={deploymentDetail ? `${deploymentDetail.ready_replicas ?? 0}/${deploymentDetail.replicas} ready` : undefined}
  >
    {#snippet actions()}
      <Button size="sm" disabled={isDeleted || !deploymentDetail || $detailReadOnly} onclick={askScale}>Scale…</Button>
      <Button size="sm" disabled={isDeleted || isRestarting || $detailReadOnly} loading={isRestarting} onclick={askRestart}>Restart…</Button>
    {/snippet}
  </DetailToolbar>

  <DetailTabs
    tabs={[
      { id: 'overview', label: 'Overview' },
      { id: 'pods', label: 'Pods' },
      { id: 'events', label: 'Events' },
      { id: 'yaml', label: 'YAML' },
    ]}
    active={activeTab}
    onchange={(id) => handleTabChange(id as TabType)}
  />

  <!-- Tombstone Banner -->
  {#if isDeleted}
    <div class="flex items-center justify-between px-4 py-3 bg-accent-warning/10 border-b border-accent-warning/30">
      <div class="flex items-center gap-3">
        <svg class="w-5 h-5 text-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div>
          <span class="font-medium text-warning">This deployment no longer exists</span>
          {#if lastSeen}
            <span class="text-text-muted text-base ml-2">Last seen: {lastSeen}</span>
          {/if}
        </div>
      </div>
      <span class="text-xs text-text-muted bg-bg-tertiary px-2 py-1 rounded-sm">Read-only</span>
    </div>
  {/if}

  <!-- Tab content -->
  <div class="flex-1 overflow-auto">
    {#if isLoading}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-8 h-8 text-accent-text mx-auto mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <p class="text-text-muted">Loading deployment details...</p>
        </div>
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-danger mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-muted">{error}</p>
          <button onclick={loadDeploymentDetail} class="mt-4 px-4 py-2 bg-btn text-on-btn rounded-lg hover:opacity-90">
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
          <button onclick={loadDeploymentDetail} class="mt-4 px-4 py-2 bg-btn text-on-btn rounded-lg hover:opacity-90">
            Retry
          </button>
        </div>
      </div>
    {:else if activeTab === 'overview'}
      <div class="d-body">
        <!-- Status Section -->
        <section>
          <h2 class="d-sec-title">Status</h2>
          <dl class="d-props">
            <div class="d-prop"><dt>Ready</dt><dd><span class="text-text-primary">{deploymentDetail.ready_replicas}/{deploymentDetail.replicas}</span></dd></div>
            <div class="d-prop"><dt>Up-to-date</dt><dd><span class="text-text-primary">{deploymentDetail.updated_replicas}</span></dd></div>
            <div class="d-prop"><dt>Available</dt><dd><span class="text-text-primary">{deploymentDetail.available_replicas}</span></dd></div>
          </dl>
        </section>

        <!-- Spec Section -->
        <section>
          <h2 class="d-sec-title">Spec</h2>
          <dl class="d-props">
            <div class="d-prop"><dt>Strategy</dt><dd><span class="text-base text-text-primary">{deploymentDetail.strategy}</span></dd></div>
            <div class="d-prop"><dt>Min Ready Seconds</dt><dd><span class="text-base text-text-primary">{deploymentDetail.min_ready_seconds}</span></dd></div>
            <div class="d-prop"><dt>Revision History</dt><dd><span class="text-base text-text-primary">{deploymentDetail.revision_history_limit ?? 'Unlimited'}</span></dd></div>
          </dl>
        </section>

        <!-- Container Images -->
        <section>
          <h2 class="d-sec-title">Container Images</h2>
          <div class="d-frame p-3">
            {#if deploymentDetail.container_images.length > 0}
              <div class="space-y-2">
                {#each deploymentDetail.container_images as image}
                  <code class="block text-base text-text-secondary break-all">{image}</code>
                {/each}
              </div>
            {:else}
              <p class="text-text-muted text-base">No container images</p>
            {/if}
          </div>
        </section>

        <!-- Conditions -->
        <ConditionsTable conditions={deploymentDetail.conditions ?? []} />

        <!-- Labels & Selector -->
        <section class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h2 class="d-sec-title">Labels</h2>
            <div class="d-frame p-3">
              {#if Object.keys(deploymentDetail.labels).length > 0}
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(deploymentDetail.labels) as [key, value]}
                    <span class="text-xs bg-bg-tertiary px-2 py-1 rounded-sm text-text-secondary">
                      {key}={value}
                    </span>
                  {/each}
                </div>
              {:else}
                <p class="text-text-muted text-base">No labels</p>
              {/if}
            </div>
          </div>
          <div>
            <h2 class="d-sec-title">Selector</h2>
            <div class="d-frame p-3">
              {#if Object.keys(deploymentDetail.selector).length > 0}
                <div class="flex flex-wrap gap-2">
                  {#each Object.entries(deploymentDetail.selector) as [key, value]}
                    <span class="text-xs bg-accent-primary/10 text-accent-text px-2 py-1 rounded-sm">
                      {key}={value}
                    </span>
                  {/each}
                </div>
              {:else}
                <p class="text-text-muted text-base">No selector</p>
              {/if}
            </div>
          </div>
        </section>
      </div>

    {:else if activeTab === 'pods'}
      <div class="p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-[18px]/7 font-semibold text-text-primary">Pods ({deploymentPods.length})</h2>
          <button
            onclick={loadDeploymentPods}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded-sm hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        {#if deploymentPods.length > 0}
          <div class="bg-bg-secondary rounded-lg overflow-hidden">
            <table class="w-full text-base">
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
                    <td class="px-4 py-2 text-accent-text hover:underline">{pod.name}</td>
                    <td class="px-4 py-2">
                      <span class="text-xs px-2 py-0.5 rounded-sm {getStatusColor(pod.status)}">{pod.status}</span>
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
      <EventsTable events={deploymentEvents} resourceType="Deployment" isDeleted={isDeleted} onRefresh={loadDeploymentEvents} />

    {:else if activeTab === 'yaml'}
      <YamlEditorPanel
        yaml={deploymentYaml}
        context={context}
        name={name}
        namespace={namespace}
        resourceType="Deployment"
        isDeleted={isDeleted}
        onRefresh={loadDeploymentYaml}
        onApplySuccess={loadDeploymentDetail}
      />
    {/if}
  </div>
</div>

{#snippet scaleBody()}
  <label class="block text-sm text-secondary">
    Replicas
    <span class="mt-1.5 flex items-center gap-2">
      <Button size="sm" aria-label="One fewer" onclick={() => (scaleReplicas = Math.max(0, scaleReplicas - 1))}>−</Button>
      <input
        type="number"
        min="0"
        step="1"
        bind:value={scaleReplicas}
        aria-invalid={!replicasValid}
        class="w-24 h-8 px-2.5 rounded-md bg-app border border-line-strong text-sm font-mono text-primary text-center"
      />
      <Button size="sm" aria-label="One more" onclick={() => (scaleReplicas = scaleReplicas + 1)}>+</Button>
      <span class="text-muted text-xs">now {deploymentDetail?.replicas ?? 0}</span>
    </span>
  </label>
{/snippet}

{#if confirmAction === 'scale'}
  <ConfirmDialog
    open={true}
    title={`Scale ${name}`}
    resource={{ kind: 'Deployment', name, namespace, context }}
    confirmLabel="Scale"
    body={scaleBody}
    confirmDisabled={!replicasValid}
    successMessage={`Scaling ${name} to ${scaleReplicas}`}
    onConfirm={doScale}
    onClose={() => (confirmAction = null)}
  />
{:else if confirmAction === 'restart'}
  <ConfirmDialog
    open={true}
    title={`Restart ${name}?`}
    resource={{ kind: 'Deployment', name, namespace, context }}
    confirmLabel="Restart"
    body="Pods are replaced one by one (rolling restart). Requests on the old pods may be cut off."
    successMessage={`Restarting ${name}`}
    onConfirm={doRestart}
    onClose={() => (confirmAction = null)}
  />
{/if}
