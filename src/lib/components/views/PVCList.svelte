<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, quantityValue, type Column } from '../ui/data-table';
  import { verdictHealth } from '../../utils/status-words';
  import { poll } from '../../utils/poll';
  import {
    pvcs,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadPVCs,
    resourceState,
    type PersistentVolumeClaimInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('pvcs');
  const health = verdictHealth<PersistentVolumeClaimInfo>();

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadPVCs(ns));
  });

  const columns: Column<PersistentVolumeClaimInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    {
      id: 'status',
      label: 'Status',
      width: '14%',
      sortKey: (p) => health(p).rank,
      cell: statusCell,
      title: (p) => `${p.verdict.reason} (${p.verdict.ruleId})`,
    },
    { id: 'volume', label: 'Volume', sortKey: 'volume', mono: true, width: '20%', hideBelow: BP.lg },
    { id: 'capacity', label: 'Capacity', sortKey: (p) => quantityValue(p.capacity), numeric: true, width: '84px' },
    { id: 'access_modes', label: 'Access', sortKey: (p) => p.access_modes.join(','), width: '10%', hideBelow: BP.lg },
    { id: 'storage_class', label: 'Storage class', sortKey: 'storage_class', width: '14%', hideBelow: BP.wide },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(p: PersistentVolumeClaimInfo)}
  {@const h = health(p)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="Persistent Volume Claims"
  viewId="pvcs"
  itemLabel="claims"
  items={$pvcs}
  loadState={$loadState}
  {columns}
  rowKey={(p) => `${p.namespace}/${p.name}`}
  filterFields={['name', 'namespace', 'status', 'volume', 'storage_class']}
  {health}
  segmentLabels={{ failed: 'Lost', warning: 'Pending', healthy: 'Bound' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  resourceType="pvc"
  onRefresh={() => loadPVCs($selectedNamespace)}
/>
