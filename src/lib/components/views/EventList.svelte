<script lang="ts">
  // Events: warnings first, then newest (spec PR7a). Normal events are
  // neutral, not healthy. The watch is started globally in App.svelte so
  // Warning notifications fire from any view; this view only reads the
  // shared store. Click / Enter opens the involved object's detail window.
  import { invoke } from '@tauri-apps/api/core';
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import CustomSelect from '../ui/CustomSelect.svelte';
  import { BP, type Column } from '../ui/data-table';
  import { eventHealth } from '../../utils/status-words';
  import { parseAge } from '../../utils/sort';
  import { toast } from '../../stores/toast';
  import {
    clusterEvents,
    currentContext,
    selectedNamespace,
    startEventWatch,
    resourceState,
    type ClusterEventInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('events');
  const health = (e: ClusterEventInfo) => eventHealth(e.type);

  let reasonFilter = $state('all');

  const reasonOptions = $derived([
    { value: 'all', label: 'All reasons' },
    ...[...new Set($clusterEvents.map((e) => e.reason).filter(Boolean))]
      .sort()
      .map((r) => ({ value: r, label: r.length > 18 ? `${r.slice(0, 18)}…` : r })),
  ]);

  // A reason that ages out of the list falls back to all reasons.
  $effect(() => {
    if (reasonFilter !== 'all' && !$clusterEvents.some((e) => e.reason === reasonFilter)) reasonFilter = 'all';
  });

  // Rank first (warnings before normal), then newest. Ages are whole seconds,
  // far below the rank step, so they only break ties within a rank.
  const typeOrder = (e: ClusterEventInfo) => health(e).rank * 1e10 + parseAge(e.age);

  const columns: Column<ClusterEventInfo>[] = [
    { id: 'type', label: 'Type', width: '110px', sortKey: typeOrder, cell: typeCell },
    { id: 'reason', label: 'Reason', sortKey: 'reason', primary: true, width: '18%' },
    {
      id: 'object',
      label: 'Object',
      sortKey: (e) => `${e.involved_kind}/${e.involved_object}`,
      mono: true,
      width: '22%',
      cell: objectCell,
      title: (e) => `${e.involved_kind}/${e.involved_object}`,
    },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '12%', hideBelow: BP.lg },
    { id: 'message', label: 'Message', hideBelow: BP.md },
    { id: 'count', label: 'Count', sortKey: 'count', numeric: true, width: '64px', hideBelow: BP.wide },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];

  // K8s kind -> detail window resource type.
  const kindToResourceType: Record<string, string> = {
    Pod: 'pod',
    Deployment: 'deployment',
    StatefulSet: 'statefulset',
    DaemonSet: 'daemonset',
    ReplicaSet: 'replicaset',
    Job: 'job',
    CronJob: 'cronjob',
    Service: 'service',
    Ingress: 'ingress',
    ConfigMap: 'configmap',
    Secret: 'secret',
    PersistentVolume: 'pv',
    PersistentVolumeClaim: 'pvc',
    Node: 'node',
    Namespace: 'namespace',
    ServiceAccount: 'serviceaccount',
    HorizontalPodAutoscaler: 'hpa',
    NetworkPolicy: 'networkpolicy',
  };

  function openObject(e: ClusterEventInfo) {
    const resourceType = kindToResourceType[e.involved_kind];
    if (!resourceType || !e.involved_object) {
      toast(`No detail view for ${e.involved_kind || 'this object'}`);
      return;
    }
    invoke('open_resource_detail', {
      resourceType,
      name: e.involved_object,
      namespace: e.namespace || '',
      context: $currentContext,
    }).catch((err) => console.error('Failed to open resource detail:', err));
  }
</script>

{#snippet typeCell(e: ClusterEventInfo)}
  {@const h = health(e)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

{#snippet objectCell(e: ClusterEventInfo)}<span class="text-muted">{e.involved_kind}/</span>{e.involved_object}{/snippet}

{#snippet actions()}
  <div class="w-40">
    <CustomSelect value={reasonFilter} options={reasonOptions} placeholder="Reason…" onchange={(v) => (reasonFilter = v)} />
  </div>
{/snippet}

<ListView
  title="Events"
  viewId="events"
  itemLabel="events"
  items={$clusterEvents}
  loadState={$loadState}
  {columns}
  rowKey={(e) => `${e.namespace}/${e.name}`}
  filterFields={['reason', 'message', 'involved_object', 'involved_kind', 'namespace']}
  {health}
  segmentLabels={{ warning: 'Warning', neutral: 'Normal' }}
  defaultSort={{ id: 'type', dir: 'asc' }}
  onOpen={openObject}
  onRefresh={() => startEventWatch($selectedNamespace)}
  copyText={(e) => e.message}
  rowFilter={reasonFilter === 'all' ? undefined : (e) => e.reason === reasonFilter}
  onClearFilter={() => (reasonFilter = 'all')}
  {actions}
/>
