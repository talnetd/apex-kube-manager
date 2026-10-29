<script lang="ts">
  import { guard, detailReadOnly } from '../../stores/detail';
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import YamlEditorPanel from '../ui/YamlEditorPanel.svelte';
  import MetadataSection from '../ui/MetadataSection.svelte';
  import EventsTable from '../ui/EventsTable.svelte';
  import ConditionsTable from '../ui/ConditionsTable.svelte';
  import CustomSelect from '../ui/CustomSelect.svelte';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import { settings } from '../../stores/settings';
  import { resolveProd } from '../../stores/production';
  import Button from '../ui/Button.svelte';
  import DetailToolbar from './kit/DetailToolbar.svelte';
  import DetailTabs from './kit/DetailTabs.svelte';
  import { parseCrType } from '../../utils/resources';

  interface Props {
    resourceType: string;
    context: string;
    namespace: string;
    name: string;
  }

  let { resourceType, context, namespace, name }: Props = $props();

  // Resource configuration - defines tabs and display names for each type
  const resourceConfig: Record<string, { displayName: string; tabs: string[]; clusterScoped?: boolean; generic?: boolean }> = {
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
    // Generic kinds (src-tauri/src/resources.rs): one dispatch command set,
    // one fields + tables overview.
    role: { displayName: 'Role', tabs: ['overview', 'events', 'yaml'], generic: true },
    clusterrole: { displayName: 'ClusterRole', tabs: ['overview', 'events', 'yaml'], clusterScoped: true, generic: true },
    rolebinding: { displayName: 'RoleBinding', tabs: ['overview', 'events', 'yaml'], generic: true },
    clusterrolebinding: { displayName: 'ClusterRoleBinding', tabs: ['overview', 'events', 'yaml'], clusterScoped: true, generic: true },
    storageclass: { displayName: 'StorageClass', tabs: ['overview', 'events', 'yaml'], clusterScoped: true, generic: true },
    endpointslice: { displayName: 'EndpointSlice', tabs: ['overview', 'events', 'yaml'], generic: true },
    ingressclass: { displayName: 'IngressClass', tabs: ['overview', 'events', 'yaml'], clusterScoped: true, generic: true },
    pdb: { displayName: 'PodDisruptionBudget', tabs: ['overview', 'events', 'yaml'], generic: true },
    resourcequota: { displayName: 'ResourceQuota', tabs: ['overview', 'events', 'yaml'], generic: true },
    limitrange: { displayName: 'LimitRange', tabs: ['overview', 'events', 'yaml'], generic: true },
    priorityclass: { displayName: 'PriorityClass', tabs: ['overview', 'events', 'yaml'], clusterScoped: true, generic: true },
    crd: { displayName: 'CustomResourceDefinition', tabs: ['overview', 'events', 'yaml'], clusterScoped: true, generic: true },
  };

  // Custom resources carry their API coordinates in the type id.
  const crTarget = $derived(parseCrType(resourceType));
  const config = $derived(
    crTarget
      ? { displayName: crTarget.kind, tabs: ['yaml', 'overview', 'events'], clusterScoped: !crTarget.namespaced, generic: true }
      : resourceConfig[resourceType] || { displayName: resourceType, tabs: ['overview', 'events', 'yaml'] }
  );

  /** Command and args for detail/yaml/events, per kind family. */
  function call<T>(what: 'detail' | 'yaml' | 'events'): Promise<T> {
    const ns = config.clusterScoped ? null : namespace;
    if (crTarget) return invoke<T>(`get_custom_resource_${what}`, { contextName: context, target: crTarget, namespace: ns, name });
    if (config.generic) return invoke<T>(`get_resource_${what}`, { contextName: context, kind: resourceType, namespace: ns, name });
    return invoke<T>(`get_${resourceType}_${what}`, getInvokeParams());
  }

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

  // Taint management state (for nodes)
  let showAddTaintModal = $state<boolean>(false);
  let newTaintKey = $state<string>('');
  let newTaintValue = $state<string>('');
  let newTaintEffect = $state<string>('NoSchedule');
  let taintActionLoading = $state<boolean>(false);
  let taintActionError = $state<string | null>(null);
  let confirmRemoveTaint = $state<{ key: string; effect: string } | null>(null);
  let confirmAddTaint = $state<{ key: string; value: string; effect: string } | null>(null);

  // Cordon state (for nodes)
  let cordonLoading = $state<boolean>(false);
  let showCordonConfirm = $state<boolean>(false);

  // Helper to format label selector maps as strings
  function formatLabels(labels: Record<string, string> | null | undefined): string {
    if (!labels || Object.keys(labels).length === 0) {
      return '<all>';
    }
    return Object.entries(labels).map(([k, v]) => `${k}=${v}`).join(', ');
  }

  // Check if a label selector has any labels
  function hasLabels(labels: Record<string, string> | null | undefined): boolean {
    return labels != null && Object.keys(labels).length > 0;
  }

  // Sync onMount so the cleanup runs (an async one's return is ignored).
  onMount(() => {
    // Custom resources have no typed overview: open on YAML.
    if (crTarget) activeTab = 'yaml';
    loadDetail();
    existenceInterval = setInterval(checkExists, 5000);
    return () => {
      if (existenceInterval) clearInterval(existenceInterval);
    };
  });

  async function checkExists() {
    if (isDeleted) return;
    try {
      await call('detail');
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
      detail = await call<Record<string, any>>('detail');
      // YAML-first types (custom resources) load their first tab now.
      if (activeTab === 'yaml' && !yaml) loadYaml();
    } catch (e) {
      error = String(e);
    } finally {
      isLoading = false;
    }
  }

  async function loadYaml() {
    try {
      yaml = await call<string>('yaml');
    } catch (e) {
      yaml = `Error loading YAML: ${e}`;
    }
  }

  async function loadEvents() {
    try {
      events = await call<any[]>('events');
    } catch (e) {
      events = [];
    }
  }

  async function loadPods() {
    try {
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
      case 'Running': return 'text-success bg-accent-success/10';
      case 'Succeeded': return 'text-accent-text bg-accent-primary/10';
      case 'Pending': return 'text-warning bg-accent-warning/10';
      case 'Failed': return 'text-danger bg-accent-error/10';
      default: return 'text-text-muted bg-bg-tertiary';
    }
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

  // Taint validation functions
  function validateTaintKey(key: string): string | null {
    if (!key.trim()) {
      return 'Key is required';
    }

    const trimmed = key.trim();

    // Check for prefix/name format
    const parts = trimmed.split('/');
    if (parts.length > 2) {
      return 'Key can only have one "/" separator';
    }

    const name = parts.length === 2 ? parts[1] : parts[0];
    const prefix = parts.length === 2 ? parts[0] : null;

    // Validate name (required)
    if (!name) {
      return 'Key name is required after prefix';
    }
    if (name.length > 63) {
      return 'Key name must be 63 characters or less';
    }
    if (!/^[a-zA-Z0-9]([a-zA-Z0-9._-]*[a-zA-Z0-9])?$|^[a-zA-Z0-9]$/.test(name)) {
      return 'Key name must start/end with alphanumeric, can contain -, _, .';
    }

    // Validate prefix (optional)
    if (prefix) {
      if (prefix.length > 253) {
        return 'Key prefix must be 253 characters or less';
      }
      if (!/^[a-z0-9]([-a-z0-9]*[a-z0-9])?(\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*$/.test(prefix)) {
        return 'Key prefix must be a valid DNS subdomain';
      }
    }

    return null;
  }

  function validateTaintValue(value: string): string | null {
    if (!value.trim()) {
      return null; // Value is optional
    }

    const trimmed = value.trim();
    if (trimmed.length > 63) {
      return 'Value must be 63 characters or less';
    }
    if (!/^[a-zA-Z0-9]([a-zA-Z0-9._-]*[a-zA-Z0-9])?$|^[a-zA-Z0-9]$/.test(trimmed)) {
      return 'Value must start/end with alphanumeric, can contain -, _, .';
    }

    return null;
  }

  // Taint management functions (for nodes)
  async function addTaint() {
    if (!guard('taint')) return;
    // Validate key
    const keyError = validateTaintKey(newTaintKey);
    if (keyError) {
      taintActionError = keyError;
      return;
    }

    // Validate value
    const valueError = validateTaintValue(newTaintValue);
    if (valueError) {
      taintActionError = valueError;
      return;
    }

    // NoExecute evicts every pod without a matching toleration: confirm it
    // (danger). Other effects only confirm on a production context.
    if (newTaintEffect === 'NoExecute' || resolveProd(context, $settings.prodOverrides)) {
      taintActionError = null;
      confirmAddTaint = { key: newTaintKey.trim(), value: newTaintValue.trim(), effect: newTaintEffect };
      return;
    }
    await runAddTaint();
  }

  async function runAddTaint() {
    try {
      taintActionLoading = true;
      taintActionError = null;
      await invoke('add_node_taint', {
        contextName: context,
        name: name,
        key: newTaintKey.trim(),
        value: newTaintValue.trim() || null,
        effect: newTaintEffect,
      });
      // Refresh detail to show updated taints
      await loadDetail();
      // Reset form
      showAddTaintModal = false;
      newTaintKey = '';
      newTaintValue = '';
      newTaintEffect = 'NoSchedule';
    } catch (e) {
      taintActionError = String(e);
    } finally {
      taintActionLoading = false;
    }
  }

  async function removeTaint(key: string, effect: string) {
    if (!guard('taint')) return;
    try {
      taintActionLoading = true;
      taintActionError = null;
      await invoke('remove_node_taint', {
        contextName: context,
        name: name,
        key: key,
        effect: effect,
      });
      // Refresh detail to show updated taints
      await loadDetail();
      confirmRemoveTaint = null;
    } catch (e) {
      taintActionError = String(e);
    } finally {
      taintActionLoading = false;
    }
  }

  // Removing a taint only re-allows scheduling (it never evicts), so it is a
  // plain action; on production it still confirms.
  function handleRemoveTaintClick(key: string, effect: string) {
    if (resolveProd(context, $settings.prodOverrides)) {
      confirmRemoveTaint = { key, effect };
    } else {
      removeTaint(key, effect);
    }
  }

  function getEffectDescription(effect: string): string {
    switch (effect) {
      case 'NoSchedule': return 'Pods will not be scheduled on this node';
      case 'PreferNoSchedule': return 'Scheduler will try to avoid placing pods on this node';
      case 'NoExecute': return 'Existing pods will be evicted, new pods will not be scheduled';
      default: return '';
    }
  }

  // Confirmed actions throw on failure: ConfirmDialog shows the error.
  async function doCordon() {
    if (!guard('cordon')) throw new Error('Switch back to this context to cordon');
    await invoke('cordon_node', { contextName: context, name });
    await loadDetail();
  }
  // From the ConfirmDialog: re-guard (the app may have switched) and throw so
  // the dialog shows the error.
  async function doAddTaint() {
    if (!guard('taint')) throw new Error('Switch back to this context to change taints');
    const t = confirmAddTaint;
    if (!t) return;
    await invoke('add_node_taint', { contextName: context, name, key: t.key, value: t.value || null, effect: t.effect });
    await loadDetail();
    showAddTaintModal = false;
    newTaintKey = '';
    newTaintValue = '';
    newTaintEffect = 'NoSchedule';
  }

  async function doRemoveTaint(key: string, effect: string) {
    if (!guard('taint')) throw new Error('Switch back to this context to change taints');
    await invoke('remove_node_taint', { contextName: context, name, key, effect });
    await loadDetail();
  }

  // Uncordon (for nodes); cordon goes through ConfirmDialog (doCordon).

  async function uncordonNode() {
    if (!guard('cordon')) return;
    try {
      cordonLoading = true;
      await invoke('uncordon_node', {
        contextName: context,
        name: name,
      });
      await loadDetail();
    } catch (e) {
      console.error('Failed to uncordon node:', e);
    } finally {
      cordonLoading = false;
    }
  }
</script>

<div class="h-full flex flex-col">
  {#snippet nodeActions()}
      {#if detail}
        {#if detail.unschedulable}
          <Button size="sm" loading={cordonLoading} disabled={isDeleted || $detailReadOnly} onclick={uncordonNode}>Uncordon</Button>
        {:else}
          <Button size="sm" variant="danger" disabled={isDeleted || $detailReadOnly} onclick={() => guard('cordon') && (showCordonConfirm = true)}>Cordon…</Button>
        {/if}
      {/if}
  {/snippet}
  <DetailToolbar verdict={detail?.verdict} actions={resourceType === 'node' ? nodeActions : undefined} />

  <DetailTabs tabs={config.tabs.map((t) => ({ id: t, label: getTabLabel(t) }))} active={activeTab} onchange={handleTabChange} />

  <!-- Deleted Banner -->
  {#if isDeleted}
    <div class="flex items-center justify-between px-4 py-3 bg-accent-warning/10 border-b border-accent-warning/30">
      <div class="flex items-center gap-3">
        <svg class="w-5 h-5 text-warning" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <div>
          <span class="font-medium text-warning">This {config.displayName} no longer exists</span>
          {#if lastSeen}
            <span class="text-text-muted text-base ml-2">Last seen: {lastSeen}</span>
          {/if}
        </div>
      </div>
      <span class="text-xs text-text-muted bg-bg-tertiary px-2 py-1 rounded-sm">Read-only</span>
    </div>
  {/if}

  <!-- Main Content: the panel the active tab controls -->
  <div class="flex-1 overflow-auto" role="tabpanel" id={`panel-${activeTab}`} aria-labelledby={`tab-${activeTab}`} tabindex="0">
    {#if isLoading}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-8 h-8 text-accent-text mx-auto mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <p class="text-text-muted">Loading {config.displayName} details...</p>
        </div>
      </div>
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-16 h-16 text-danger mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
          </svg>
          <p class="text-text-muted">{error}</p>
          <button onclick={loadDetail} class="mt-4 px-4 py-2 bg-btn text-on-btn rounded-lg hover:opacity-90">Retry</button>
        </div>
      </div>
    {:else if !detail}
      <div class="flex items-center justify-center h-full">
        <p class="text-text-muted">No {config.displayName} data loaded</p>
      </div>

    <!-- OVERVIEW TAB -->
    {:else if activeTab === 'overview'}
      <div class="d-body">
        <!-- Basic Info - Common to all -->
        <section>
          <h2 class="d-sec-title">{config.displayName} Info</h2>
          <dl class="d-props">
            <div class="d-prop"><dt>Name</dt><dd><span class="text-base text-text-primary font-mono truncate block">{detail.name}</span></dd></div>
            {#if detail.namespace}
              <div class="d-prop"><dt>Namespace</dt><dd><span class="text-base text-text-primary">{detail.namespace}</span></dd></div>
            {/if}
            <div class="d-prop"><dt>UID</dt><dd><span class="text-xs text-text-secondary font-mono truncate block">{detail.uid}</span></dd></div>
            <div class="d-prop"><dt>Created</dt><dd><span class="text-base text-text-primary">{detail.creation_timestamp}</span></dd></div>
          </dl>
        </section>

        <!-- Resource-specific Overview Sections -->

        <!-- DEPLOYMENT -->
        {#if resourceType === 'deployment'}
          <section>
            <h2 class="d-sec-title">Replica Status</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Desired</dt><dd><span class="text-text-primary">{detail.replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Ready</dt><dd><span class="text-success">{detail.ready_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Available</dt><dd><span class="text-primary">{detail.available_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Up-to-date</dt><dd><span class="text-text-primary">{detail.updated_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Unavailable</dt><dd><span class="{(detail.unavailable_replicas ?? 0) > 0 ? 'text-danger' : 'text-text-primary'}">{detail.unavailable_replicas ?? 0}</span></dd></div>
            </dl>
          </section>
          {#if detail.strategy}
            <section>
              <h2 class="d-sec-title">Strategy</h2>
              <div class="d-frame p-3">
                <span class="text-base px-2 py-0.5 rounded-sm bg-accent-primary/10 text-accent-text">{detail.strategy}</span>
              </div>
            </section>
          {/if}

        <!-- STATEFULSET -->
        {:else if resourceType === 'statefulset'}
          <section>
            <h2 class="d-sec-title">Replica Status</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Desired</dt><dd><span class="text-text-primary">{detail.replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Ready</dt><dd><span class="text-success">{detail.ready_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Current</dt><dd><span class="text-primary">{detail.current_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Updated</dt><dd><span class="text-text-primary">{detail.updated_replicas ?? 0}</span></dd></div>
            </dl>
          </section>
          {#if detail.service_name}
            <section>
              <h2 class="d-sec-title">Service</h2>
              <div class="d-frame p-3">
                <span class="text-base text-accent-text font-mono">{detail.service_name}</span>
              </div>
            </section>
          {/if}

        <!-- DAEMONSET -->
        {:else if resourceType === 'daemonset'}
          <section>
            <h2 class="d-sec-title">Scheduling Status</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Desired</dt><dd><span class="text-text-primary">{detail.desired_number_scheduled ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Current</dt><dd><span class="text-primary">{detail.current_number_scheduled ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Ready</dt><dd><span class="text-success">{detail.number_ready ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Available</dt><dd><span class="text-success">{detail.number_available ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Misscheduled</dt><dd><span class="{(detail.number_misscheduled ?? 0) > 0 ? 'text-danger' : 'text-text-primary'}">{detail.number_misscheduled ?? 0}</span></dd></div>
            </dl>
          </section>

        <!-- REPLICASET -->
        {:else if resourceType === 'replicaset'}
          <section>
            <h2 class="d-sec-title">Replica Status</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Desired</dt><dd><span class="text-text-primary">{detail.replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Ready</dt><dd><span class="text-success">{detail.ready_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Available</dt><dd><span class="text-primary">{detail.available_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Fully Labeled</dt><dd><span class="text-text-primary">{detail.fully_labeled_replicas ?? 0}</span></dd></div>
            </dl>
          </section>
          {#if detail.owner_references && detail.owner_references.length > 0}
            <section>
              <h2 class="d-sec-title">Owner</h2>
              <div class="d-frame p-3">
                {#each detail.owner_references as owner}
                  <div class="flex items-center gap-2">
                    <span class="text-xs px-2 py-0.5 rounded-sm bg-accent-primary/10 text-accent-text">{owner.kind}</span>
                    <span class="text-base text-text-primary font-mono">{owner.name}</span>
                  </div>
                {/each}
              </div>
            </section>
          {/if}

        <!-- JOB -->
        {:else if resourceType === 'job'}
          <section>
            <h2 class="d-sec-title">Job Status</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Status</dt><dd><span class="text-base px-2 py-0.5 rounded-sm {detail.completion_time ? 'text-success bg-accent-success/10' : detail.active > 0 ? 'text-primary bg-accent-primary/10' : 'text-warning bg-accent-warning/10'}">
                  {detail.completion_time ? 'Complete' : detail.active > 0 ? 'Running' : 'Pending'}
                </span></dd></div>
              <div class="d-prop"><dt>Active</dt><dd><span class="text-primary">{detail.active ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Succeeded</dt><dd><span class="text-success">{detail.succeeded ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Failed</dt><dd><span class="{(detail.failed ?? 0) > 0 ? 'text-danger' : 'text-text-primary'}">{detail.failed ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Completions</dt><dd><span class="text-base text-text-primary">{detail.succeeded ?? 0}/{detail.completions ?? '∞'}</span></dd></div>
            </dl>
          </section>
          <section>
            <h2 class="d-sec-title">Configuration</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Parallelism</dt><dd><span class="text-base text-text-primary">{detail.parallelism ?? 1}</span></dd></div>
              <div class="d-prop"><dt>Backoff Limit</dt><dd><span class="text-base text-text-primary">{detail.backoff_limit ?? 6}</span></dd></div>
              <div class="d-prop"><dt>Completion Mode</dt><dd><span class="text-base text-text-primary">{detail.completion_mode || 'NonIndexed'}</span></dd></div>
              <div class="d-prop"><dt>TTL After Finished</dt><dd><span class="text-base text-text-primary">{detail.ttl_seconds_after_finished ? `${detail.ttl_seconds_after_finished}s` : 'Never'}</span></dd></div>
            </dl>
          </section>

        <!-- CRONJOB -->
        {:else if resourceType === 'cronjob'}
          <section>
            <h2 class="d-sec-title">Schedule</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Schedule</dt><dd><code class="text-base text-primary">{detail.schedule}</code></dd></div>
              <div class="d-prop"><dt>Suspended</dt><dd><span class="text-base px-2 py-0.5 rounded-sm {detail.suspend ? 'text-warning bg-accent-warning/10' : 'text-success bg-accent-success/10'}">
                  {detail.suspend ? 'Yes' : 'No'}
                </span></dd></div>
              <div class="d-prop"><dt>Active Jobs</dt><dd><span class="text-primary">{detail.active_jobs ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Last Schedule</dt><dd><span class="text-base text-text-primary">{detail.last_schedule_time || 'Never'}</span></dd></div>
            </dl>
          </section>
          <section>
            <h2 class="d-sec-title">Configuration</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Concurrency Policy</dt><dd><span class="text-base text-text-primary">{detail.concurrency_policy || 'Allow'}</span></dd></div>
              <div class="d-prop"><dt>Success History</dt><dd><span class="text-base text-text-primary">{detail.successful_jobs_history_limit ?? 3}</span></dd></div>
              <div class="d-prop"><dt>Failed History</dt><dd><span class="text-base text-text-primary">{detail.failed_jobs_history_limit ?? 1}</span></dd></div>
            </dl>
          </section>

        <!-- SERVICE -->
        {:else if resourceType === 'service'}
          <section>
            <h2 class="d-sec-title">Service Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Type</dt><dd><span class="text-base px-2 py-0.5 rounded-sm bg-accent-primary/10 text-primary">{detail.service_type}</span></dd></div>
              <div class="d-prop"><dt>Cluster IP</dt><dd><span class="text-base text-text-primary font-mono">{detail.cluster_ip || 'None'}</span></dd></div>
              {#if detail.external_ips && detail.external_ips.length > 0}
                <div class="d-prop"><dt>External IPs</dt><dd><span class="text-base text-text-primary font-mono">{detail.external_ips.join(', ')}</span></dd></div>
              {/if}
              {#if detail.load_balancer_ip}
                <div class="d-prop"><dt>Load Balancer IP</dt><dd><span class="text-base text-text-primary font-mono">{detail.load_balancer_ip}</span></dd></div>
              {/if}
            </dl>
          </section>
          {#if detail.ports && detail.ports.length > 0}
            <section>
              <h2 class="d-sec-title">Ports</h2>
              <div class="bg-bg-secondary rounded-lg overflow-hidden">
                <table class="w-full text-base">
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
                        <td class="px-4 py-2 text-accent-text font-mono">{port.port}</td>
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
            <h2 class="d-sec-title">Ingress Info</h2>
            <dl class="d-props">
              {#if detail.ingress_class_name}
                <div class="d-prop"><dt>Ingress Class</dt><dd><span class="text-base text-primary">{detail.ingress_class_name}</span></dd></div>
              {/if}
              {#if detail.load_balancer_ips && detail.load_balancer_ips.length > 0}
                <div class="d-prop"><dt>Load Balancer</dt><dd><span class="text-base text-text-primary font-mono">{detail.load_balancer_ips.join(', ')}</span></dd></div>
              {/if}
            </dl>
          </section>
          {#if detail.rules && detail.rules.length > 0}
            <section>
              <h2 class="d-sec-title">Rules</h2>
              <div class="space-y-3">
                {#each detail.rules as rule}
                  <div class="d-frame p-3">
                    <div class="text-base font-medium text-accent-text mb-2">{rule.host || '*'}</div>
                    {#if rule.paths && rule.paths.length > 0}
                      <div class="space-y-1">
                        {#each rule.paths as path}
                          <div class="flex items-center gap-2 text-base">
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
            <h2 class="d-sec-title">Data Summary</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Data Keys</dt><dd><span class="text-primary">{Object.keys(detail.data || {}).length}</span></dd></div>
              <div class="d-prop"><dt>Binary Keys</dt><dd><span class="text-text-primary">{(detail.binary_data_keys || []).length}</span></dd></div>
            </dl>
          </section>

        <!-- SECRET -->
        {:else if resourceType === 'secret'}
          <section>
            <h2 class="d-sec-title">Secret Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Type</dt><dd><span class="text-base text-warning">{detail.secret_type}</span></dd></div>
              <div class="d-prop"><dt>Data Keys</dt><dd><span class="text-primary">{(detail.data_keys || []).length}</span></dd></div>
            </dl>
          </section>

        <!-- NETWORKPOLICY -->
        {:else if resourceType === 'networkpolicy'}
          <section>
            <h2 class="d-sec-title">Policy Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Pod Selector</dt><dd><code class="text-base text-text-primary">{formatLabels(detail.pod_selector)}</code></dd></div>
              <div class="d-prop"><dt>Policy Types</dt><dd><div class="flex gap-2">
                  {#each (detail.policy_types || []) as ptype}
                    <span class="text-xs px-2 py-0.5 rounded-sm {ptype === 'Ingress' ? 'bg-accent-success/10 text-success' : 'bg-accent-warning/10 text-warning'}">{ptype}</span>
                  {/each}
                </div></dd></div>
            </dl>
          </section>

        <!-- HPA -->
        {:else if resourceType === 'hpa'}
          <section>
            <h2 class="d-sec-title">Autoscaling Status</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Target</dt><dd><span class="text-base text-primary">{detail.scale_target_ref}</span></dd></div>
              <div class="d-prop"><dt>Min Replicas</dt><dd><span class="text-text-primary">{detail.min_replicas ?? 1}</span></dd></div>
              <div class="d-prop"><dt>Max Replicas</dt><dd><span class="text-text-primary">{detail.max_replicas}</span></dd></div>
              <div class="d-prop"><dt>Current</dt><dd><span class="text-success">{detail.current_replicas ?? 0}</span></dd></div>
              <div class="d-prop"><dt>Desired</dt><dd><span class="text-primary">{detail.desired_replicas ?? 0}</span></dd></div>
            </dl>
          </section>

        <!-- PV -->
        {:else if resourceType === 'pv'}
          <section>
            <h2 class="d-sec-title">Volume Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Status</dt><dd><span class="text-base px-2 py-0.5 rounded-sm {detail.phase === 'Bound' ? 'text-success bg-accent-success/10' : detail.phase === 'Available' ? 'text-primary bg-accent-primary/10' : 'text-warning bg-accent-warning/10'}">{detail.phase}</span></dd></div>
              <div class="d-prop"><dt>Capacity</dt><dd><span class="text-primary">{detail.capacity}</span></dd></div>
              <div class="d-prop"><dt>Access Modes</dt><dd><span class="text-base text-text-primary">{(detail.access_modes || []).join(', ')}</span></dd></div>
              <div class="d-prop"><dt>Reclaim Policy</dt><dd><span class="text-base text-text-primary">{detail.reclaim_policy}</span></dd></div>
            </dl>
          </section>
          {#if detail.claim_ref}
            <section>
              <h2 class="d-sec-title">Bound Claim</h2>
              <div class="d-frame p-3">
                <span class="text-base text-accent-text font-mono">{detail.claim_ref}</span>
              </div>
            </section>
          {/if}

        <!-- PVC -->
        {:else if resourceType === 'pvc'}
          <section>
            <h2 class="d-sec-title">Claim Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Status</dt><dd><span class="text-base px-2 py-0.5 rounded-sm {detail.phase === 'Bound' ? 'text-success bg-accent-success/10' : 'text-warning bg-accent-warning/10'}">{detail.phase}</span></dd></div>
              <div class="d-prop"><dt>Capacity</dt><dd><span class="text-primary">{detail.capacity || 'Pending'}</span></dd></div>
              <div class="d-prop"><dt>Access Modes</dt><dd><span class="text-base text-text-primary">{(detail.access_modes || []).join(', ')}</span></dd></div>
              <div class="d-prop"><dt>Storage Class</dt><dd><span class="text-base text-text-primary">{detail.storage_class || 'Default'}</span></dd></div>
            </dl>
          </section>
          {#if detail.volume_name}
            <section>
              <h2 class="d-sec-title">Bound Volume</h2>
              <div class="d-frame p-3">
                <span class="text-base text-accent-text font-mono">{detail.volume_name}</span>
              </div>
            </section>
          {/if}

        <!-- NAMESPACE -->
        {:else if resourceType === 'namespace'}
          <section>
            <h2 class="d-sec-title">Namespace Status</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Status</dt><dd><span class="text-base px-2 py-0.5 rounded-sm {detail.phase === 'Active' ? 'text-success bg-accent-success/10' : 'text-warning bg-accent-warning/10'}">{detail.phase}</span></dd></div>
              {#if detail.finalizers && detail.finalizers.length > 0}
                <div class="d-prop"><dt>Finalizers</dt><dd><div class="flex flex-wrap gap-1">
                    {#each detail.finalizers as f}
                      <span class="text-xs bg-bg-tertiary px-2 py-0.5 rounded-sm text-text-secondary">{f}</span>
                    {/each}
                  </div></dd></div>
              {/if}
            </dl>
          </section>

        <!-- NODE -->
        {:else if resourceType === 'node'}
          <section>
            <h2 class="d-sec-title">Node Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Status</dt><dd><span class="text-base px-2 py-0.5 rounded-sm {detail.status === 'Ready' ? 'text-success bg-accent-success/10' : 'text-danger bg-accent-error/10'}">{detail.status}</span></dd></div>
              <div class="d-prop"><dt>Roles</dt><dd><span class="text-base text-text-primary">{(detail.roles || []).join(', ') || 'worker'}</span></dd></div>
              <div class="d-prop"><dt>Kubelet Version</dt><dd><span class="text-base text-text-primary">{detail.kubelet_version}</span></dd></div>
              <div class="d-prop"><dt>Internal IP</dt><dd><span class="text-base text-text-primary font-mono">{detail.internal_ip || 'N/A'}</span></dd></div>
            </dl>
          </section>
          <section>
            <h2 class="d-sec-title">System Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>OS</dt><dd><span class="text-base text-text-primary">{detail.os_image}</span></dd></div>
              <div class="d-prop"><dt>Kernel</dt><dd><span class="text-base text-text-primary">{detail.kernel_version}</span></dd></div>
              <div class="d-prop"><dt>Container Runtime</dt><dd><span class="text-base text-text-primary">{detail.container_runtime_version}</span></dd></div>
            </dl>
          </section>

          <section>
            <h2 class="d-sec-title">Scheduling</h2>
            <div class="d-frame p-3">
              <div class="flex items-center justify-between">
                <div>
                  <div class="flex items-center gap-2 mb-1">
                    {#if detail.unschedulable}
                      <span class="flex items-center gap-1.5 text-warning">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                        </svg>
                        <span class="font-medium">Cordoned</span>
                      </span>
                    {:else}
                      <span class="flex items-center gap-1.5 text-success">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <span class="font-medium">Schedulable</span>
                      </span>
                    {/if}
                  </div>
                  <p class="text-xs text-text-muted">
                    {#if detail.unschedulable}
                      New pods will not be scheduled on this node
                    {:else}
                      New pods can be scheduled on this node
                    {/if}
                  </p>
                </div>

              </div>
            </div>
          </section>

        <!-- SERVICEACCOUNT -->
        {:else if config.generic && detail.fields}
          <!-- Generic kinds: key/value fields, then small tables -->
          {#if detail.fields.length}
            <section>
              <h2 class="d-sec-title">Details</h2>
              <dl class="d-props">
                {#each detail.fields as f (f.label)}
                  <div class="d-prop"><dt>{f.label}</dt><dd>{f.value}</dd></div>
                {/each}
              </dl>
            </section>
          {/if}
          {#each detail.tables ?? [] as t (t.title)}
            <section>
              <h2 class="d-sec-title">{t.title} <span class="text-muted font-normal tabular-nums">{t.rows.length}</span></h2>
              <div class="d-frame">
                <table class="d-table">
                  <thead>
                    <tr>{#each t.columns as c}<th>{c}</th>{/each}</tr>
                  </thead>
                  <tbody>
                    {#each t.rows as row, ri (ri)}
                      <tr>{#each row as cell, ci (ci)}<td>{cell || '–'}</td>{/each}</tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            </section>
          {/each}
        {:else if resourceType === 'serviceaccount'}
          <section>
            <h2 class="d-sec-title">ServiceAccount Info</h2>
            <dl class="d-props">
              <div class="d-prop"><dt>Secrets</dt><dd><span class="text-primary">{(detail.secrets || []).length}</span></dd></div>
              <div class="d-prop"><dt>Image Pull Secrets</dt><dd><span class="text-text-primary">{(detail.image_pull_secrets || []).length}</span></dd></div>
            </dl>
          </section>
          {#if detail.secrets && detail.secrets.length > 0}
            <section>
              <h2 class="d-sec-title">Secrets</h2>
              <div class="d-frame p-3">
                <div class="space-y-2">
                  {#each detail.secrets as secret}
                    <span class="block text-base text-accent-text font-mono">{secret}</span>
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
          <h2 class="text-[18px]/7 font-semibold text-text-primary">Pods ({pods.length})</h2>
          <button
            onclick={loadPods}
            disabled={isDeleted}
            class="text-xs px-3 py-1 bg-bg-tertiary rounded-sm hover:bg-border-subtle transition-colors disabled:opacity-50"
          >
            Refresh
          </button>
        </div>
        {#if pods.length > 0}
          <div class="bg-bg-secondary rounded-lg overflow-hidden">
            <table class="w-full text-base">
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
                      <span class="text-xs px-2 py-0.5 rounded-sm {getPodStatusColor(pod.status)}">{pod.status}</span>
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
            <h2 class="text-[18px]/7 font-semibold text-text-primary">Data ({Object.keys(detail.data || {}).length} keys)</h2>
          </div>
          {#if Object.keys(detail.data || {}).length > 0}
            <div class="space-y-3">
              {#each Object.entries(detail.data || {}) as [key, value]}
                <div class="bg-bg-secondary rounded-lg overflow-hidden">
                  <button
                    onclick={() => toggleKey(key)}
                    class="w-full flex items-center justify-between px-4 py-3 hover:bg-bg-tertiary transition-colors"
                  >
                    <span class="font-mono text-base text-accent-text">{key}</span>
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
            <h2 class="text-[18px]/7 font-semibold text-text-primary">Secret Data ({(detail.data_keys || []).length} keys)</h2>
            <button
              onclick={loadSecretData}
              disabled={isDeleted}
              class="text-xs px-3 py-1 bg-bg-tertiary rounded-sm hover:bg-border-subtle transition-colors disabled:opacity-50"
            >
              {Object.keys(secretData).length > 0 ? 'Refresh' : 'Load Data'}
            </button>
          </div>
          {#if (detail.data_keys || []).length > 0}
            <div class="space-y-3">
              {#each (detail.data_keys || []) as key}
                <div class="bg-bg-secondary rounded-lg overflow-hidden">
                  <div class="flex items-center justify-between px-4 py-3">
                    <span class="font-mono text-base text-warning">{key}</span>
                    <div class="flex items-center gap-2">
                      {#if secretData[key]}
                        <button
                          onclick={() => toggleReveal(key)}
                          class="text-xs px-2 py-1 rounded-sm bg-bg-tertiary hover:bg-accent-warning/20 text-text-muted hover:text-warning transition-colors"
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
        <h2 class="d-sec-title">Ingress Rules</h2>
        {#if detail.ingress_rules && detail.ingress_rules.length > 0}
          <div class="space-y-4">
            {#each detail.ingress_rules as rule, i}
              <div class="d-frame p-3">
                <div class="text-base font-medium text-success mb-3">Rule {i + 1}</div>
                {#if rule.from && rule.from.length > 0}
                  <div class="mb-3">
                    <div class="text-xs text-text-muted uppercase mb-2">From</div>
                    <div class="space-y-2">
                      {#each rule.from as peer}
                        <div class="text-base bg-bg-tertiary rounded-sm px-3 py-2 flex flex-wrap gap-x-4 gap-y-1">
                          {#if peer.ip_block}
                            <span><span class="text-danger">cidr:</span> <code class="text-text-secondary">{peer.ip_block}</code></span>
                          {:else}
                            {#if hasLabels(peer.pod_selector) || !hasLabels(peer.namespace_selector)}
                              <span><span class="text-accent-text">pods:</span> <code class="text-text-secondary">{formatLabels(peer.pod_selector)}</code></span>
                            {/if}
                            {#if hasLabels(peer.namespace_selector)}
                              <span><span class="text-warning">namespaces:</span> <code class="text-text-secondary">{formatLabels(peer.namespace_selector)}</code></span>
                            {/if}
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                {:else}
                  <div class="text-base text-text-muted mb-3">From: All sources</div>
                {/if}
                {#if rule.ports && rule.ports.length > 0}
                  <div>
                    <div class="text-xs text-text-muted uppercase mb-2">Ports</div>
                    <div class="flex flex-wrap gap-2">
                      {#each rule.ports as port}
                        <span class="text-xs bg-bg-tertiary px-2 py-1 rounded-sm text-text-primary">{port.protocol}/{port.port}</span>
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
        <h2 class="d-sec-title">Egress Rules</h2>
        {#if detail.egress_rules && detail.egress_rules.length > 0}
          <div class="space-y-4">
            {#each detail.egress_rules as rule, i}
              <div class="d-frame p-3">
                <div class="text-base font-medium text-warning mb-3">Rule {i + 1}</div>
                {#if rule.to && rule.to.length > 0}
                  <div class="mb-3">
                    <div class="text-xs text-text-muted uppercase mb-2">To</div>
                    <div class="space-y-2">
                      {#each rule.to as peer}
                        <div class="text-base bg-bg-tertiary rounded-sm px-3 py-2 flex flex-wrap gap-x-4 gap-y-1">
                          {#if peer.ip_block}
                            <span><span class="text-danger">cidr:</span> <code class="text-text-secondary">{peer.ip_block}</code></span>
                          {:else}
                            {#if hasLabels(peer.pod_selector) || !hasLabels(peer.namespace_selector)}
                              <span><span class="text-accent-text">pods:</span> <code class="text-text-secondary">{formatLabels(peer.pod_selector)}</code></span>
                            {/if}
                            {#if hasLabels(peer.namespace_selector)}
                              <span><span class="text-warning">namespaces:</span> <code class="text-text-secondary">{formatLabels(peer.namespace_selector)}</code></span>
                            {/if}
                          {/if}
                        </div>
                      {/each}
                    </div>
                  </div>
                {:else}
                  <div class="text-base text-text-muted mb-3">To: All destinations</div>
                {/if}
                {#if rule.ports && rule.ports.length > 0}
                  <div>
                    <div class="text-xs text-text-muted uppercase mb-2">Ports</div>
                    <div class="flex flex-wrap gap-2">
                      {#each rule.ports as port}
                        <span class="text-xs bg-bg-tertiary px-2 py-1 rounded-sm text-text-primary">{port.protocol}/{port.port}</span>
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
        <h2 class="d-sec-title">Metrics</h2>
        {#if detail.metrics && detail.metrics.length > 0}
          <div class="space-y-4">
            {#each detail.metrics as metric}
              <div class="d-frame p-3">
                <div class="flex items-center justify-between mb-2">
                  <span class="text-base font-medium text-text-primary">{metric.name || metric.metric_type}</span>
                  <span class="text-xs px-2 py-0.5 rounded-sm bg-accent-primary/10 text-accent-text">{metric.metric_type}</span>
                </div>
                <div class="grid grid-cols-2 gap-4 mt-3">
                  <div>
                    <div class="text-xs text-text-muted">Current</div>
                    <span class="text-[18px]/7 font-bold text-success">{metric.current_value || 'N/A'}</span>
                  </div>
                  <div>
                    <div class="text-xs text-text-muted">Target</div>
                    <span class="text-[18px]/7 font-bold text-text-primary">{metric.target_value || 'N/A'}</span>
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
        <h2 class="d-sec-title">Resource Capacity</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="d-frame p-3">
            <h3 class="text-base font-medium text-text-primary mb-4">Capacity</h3>
            <div class="space-y-3">
              {#each Object.entries(detail.capacity || {}) as [key, value]}
                <div class="flex justify-between items-center">
                  <span class="text-base text-text-muted">{key}</span>
                  <span class="text-base text-text-primary font-mono">{value}</span>
                </div>
              {/each}
            </div>
          </div>
          <div class="d-frame p-3">
            <h3 class="text-base font-medium text-text-primary mb-4">Allocatable</h3>
            <div class="space-y-3">
              {#each Object.entries(detail.allocatable || {}) as [key, value]}
                <div class="flex justify-between items-center">
                  <span class="text-base text-text-muted">{key}</span>
                  <span class="text-base text-success font-mono">{value}</span>
                </div>
              {/each}
            </div>
          </div>
        </div>
        <!-- Taints Section -->
        <div class="mt-6">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-[18px]/7 font-semibold text-text-primary">Taints ({(detail.taints || []).length})</h2>
            <button
              onclick={() => { showAddTaintModal = true; taintActionError = null; }}
              disabled={isDeleted || taintActionLoading || $detailReadOnly}
              class="flex items-center gap-2 px-3 py-1.5 text-base bg-btn text-on-btn rounded-lg hover:opacity-90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Add Taint
            </button>
          </div>

          {#if taintActionError}
            <div class="mb-4 p-3 bg-accent-error/10 border border-accent-error/30 rounded-lg text-danger text-base">
              {taintActionError}
            </div>
          {/if}

          {#if detail.taints && detail.taints.length > 0}
            <div class="bg-bg-secondary rounded-lg overflow-hidden">
              <table class="w-full text-base">
                <thead class="bg-bg-tertiary">
                  <tr>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Key</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Value</th>
                    <th class="px-4 py-2 text-left text-xs text-text-muted uppercase">Effect</th>
                    <th class="px-4 py-2 text-right text-xs text-text-muted uppercase">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {#each detail.taints as taint}
                    <tr class="border-t border-border-subtle hover:bg-bg-tertiary">
                      <td class="px-4 py-2 text-text-primary font-mono">{taint.key}</td>
                      <td class="px-4 py-2 text-text-secondary">{taint.value || '-'}</td>
                      <td class="px-4 py-2">
                        <span class="text-xs px-2 py-0.5 rounded-sm {taint.effect === 'NoSchedule' ? 'bg-accent-error/10 text-danger' : taint.effect === 'PreferNoSchedule' ? 'bg-accent-warning/10 text-warning' : 'bg-accent-primary/10 text-accent-text'}">{taint.effect}</span>
                      </td>
                      <td class="px-4 py-2 text-right">
                        <button
                          onclick={() => handleRemoveTaintClick(taint.key, taint.effect)}
                          disabled={isDeleted || taintActionLoading}
                          class="text-xs px-2 py-1 text-danger hover:bg-accent-error/10 rounded-sm transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                          Remove
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {:else}
            <div class="bg-bg-secondary rounded-lg p-8 text-center">
              <svg class="w-12 h-12 text-text-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <p class="text-text-muted">No taints configured on this node</p>
              <p class="text-xs text-text-muted mt-1">All pods can be scheduled on this node</p>
            </div>
          {/if}
        </div>
      </div>

    <!-- Add Taint Modal -->
    {#if showAddTaintModal}
      <div role="presentation" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onclick={(e) => { if (e.target === e.currentTarget && !taintActionLoading) showAddTaintModal = false; }}>
        <div role="dialog" aria-modal="true" tabindex="-1" class="bg-bg-secondary rounded-lg border border-line-strong w-full max-w-md mx-4">
          <div class="flex items-center justify-between px-6 py-4 border-b border-border-subtle">
            <h3 class="text-[18px]/7 font-semibold text-text-primary">Add Taint</h3>
            <button aria-label="Close"
              onclick={() => showAddTaintModal = false}
              disabled={taintActionLoading}
              class="text-text-muted hover:text-text-primary disabled:opacity-50"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="p-6 space-y-4">
            {#if taintActionError}
              <div class="p-3 bg-accent-error/10 border border-accent-error/30 rounded-lg text-danger text-base">
                {taintActionError}
              </div>
            {/if}

            <div>
              <label for="taint-key" class="block text-base font-medium text-text-primary mb-1">Key *</label>
              <input
                id="taint-key"
                type="text"
                bind:value={newTaintKey}
                placeholder="e.g., dedicated, gpu, node-role"
                disabled={taintActionLoading}
                class="w-full px-3 py-2 bg-bg-primary border border-border-subtle rounded-lg text-text-primary placeholder-text-muted focus:outline-hidden focus:border-accent-primary disabled:opacity-50"
              />
            </div>

            <div>
              <label for="taint-value" class="block text-base font-medium text-text-primary mb-1">Value (optional)</label>
              <input
                id="taint-value"
                type="text"
                bind:value={newTaintValue}
                placeholder="e.g., special-user, nvidia"
                disabled={taintActionLoading}
                class="w-full px-3 py-2 bg-bg-primary border border-border-subtle rounded-lg text-text-primary placeholder-text-muted focus:outline-hidden focus:border-accent-primary disabled:opacity-50"
              />
            </div>

            <div>
              <span class="block text-base font-medium text-text-primary mb-1">Effect *</span>
              <CustomSelect
                bind:value={newTaintEffect}
                disabled={taintActionLoading}
                options={[
                  { value: 'NoSchedule', label: 'NoSchedule' },
                  { value: 'PreferNoSchedule', label: 'PreferNoSchedule' },
                  { value: 'NoExecute', label: 'NoExecute' }
                ]}
                placeholder="Select effect..."
              />
              <p class="text-xs text-text-muted mt-1">{getEffectDescription(newTaintEffect)}</p>
            </div>

            {#if newTaintEffect === 'NoExecute'}
              <div class="p-3 bg-accent-warning/10 border border-accent-warning/30 rounded-lg">
                <div class="flex items-start gap-2">
                  <svg class="w-5 h-5 text-warning shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                  <div class="text-base text-warning">
                    <strong>Warning:</strong> NoExecute will evict existing pods that don't tolerate this taint.
                  </div>
                </div>
              </div>
            {/if}
          </div>

          <div class="flex justify-end gap-3 px-6 py-4 border-t border-border-subtle bg-bg-tertiary rounded-b-lg">
            <button
              onclick={() => showAddTaintModal = false}
              disabled={taintActionLoading}
              class="px-4 py-2 text-base text-text-secondary hover:text-text-primary transition-colors disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              onclick={addTaint}
              disabled={taintActionLoading || !newTaintKey.trim()}
              class="flex items-center gap-2 px-4 py-2 text-base bg-btn text-on-btn rounded-lg hover:opacity-90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {#if taintActionLoading}
                <svg class="w-4 h-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              {/if}
              Add Taint
            </button>
          </div>
        </div>
      </div>
    {/if}

    {#if confirmAddTaint}
      {@const t = confirmAddTaint}
      <ConfirmDialog
        open={true}
        title={t.effect === 'NoExecute' ? 'Add a NoExecute taint?' : `Add taint ${t.key}?`}
        resource={{ kind: 'Node', name, context }}
        confirmLabel="Add taint"
        tone={t.effect === 'NoExecute' ? 'danger' : 'default'}
        body={t.effect === 'NoExecute'
          ? `${t.key}${t.value ? '=' + t.value : ''}:NoExecute evicts every pod on this node that doesn't tolerate it, right away.`
          : `${t.key}${t.value ? '=' + t.value : ''}:${t.effect}. New pods without a matching toleration won't be scheduled here.`}
        successMessage={`Added ${t.key}:${t.effect}`}
        onConfirm={doAddTaint}
        onClose={() => (confirmAddTaint = null)}
      />
    {/if}

    {#if confirmRemoveTaint}
      {@const t = confirmRemoveTaint}
      <ConfirmDialog
        open={true}
        title={`Remove taint ${t.key}?`}
        resource={{ kind: 'Node', name, context }}
        confirmLabel="Remove taint"
        body={`${t.key}:${t.effect}. Pods without a matching toleration may be scheduled on this node again.`}
        successMessage={`Removed ${t.key}:${t.effect}`}
        onConfirm={() => doRemoveTaint(t.key, t.effect)}
        onClose={() => (confirmRemoveTaint = null)}
      />
    {/if}

    {#if showCordonConfirm}
      <ConfirmDialog
        open={true}
        title={`Cordon ${name}?`}
        resource={{ kind: 'Node', name, context }}
        confirmLabel="Cordon"
        body="New pods won't be scheduled on this node. Running pods keep running, and you can uncordon it later."
        successMessage={`Cordoned ${name}`}
        onConfirm={doCordon}
        onClose={() => (showCordonConfirm = false)}
      />
    {/if}

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
        name={name}
        namespace={config.clusterScoped ? undefined : namespace}
        resourceType={config.displayName}
        isDeleted={isDeleted}
        onRefresh={loadYaml}
        onApplySuccess={loadDetail}
      />
    {/if}
  </div>
</div>

