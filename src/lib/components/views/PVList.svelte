<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, quantityValue, type Column } from '../ui/data-table';
  import { pvHealth } from '../../utils/status-words';
  import {
    pvList,
    type PersistentVolumeInfo,
  } from '../../stores/kubernetes';

  const health = (pv: PersistentVolumeInfo) => pvHealth(pv.status);

  const columns: Column<PersistentVolumeInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'status', label: 'Status', width: '14%', sortKey: (pv) => health(pv).rank, cell: statusCell },
    { id: 'capacity', label: 'Capacity', sortKey: (pv) => quantityValue(pv.capacity), numeric: true, width: '84px' },
    { id: 'access_modes', label: 'Access', sortKey: (pv) => pv.access_modes.join(','), width: '12%', hideBelow: BP.lg },
    { id: 'reclaim_policy', label: 'Reclaim', sortKey: 'reclaim_policy', width: '84px', hideBelow: BP.lg },
    { id: 'claim', label: 'Claim', sortKey: 'claim', mono: true, width: '22%', hideBelow: BP.wide },
    { id: 'storage_class', label: 'Storage class', sortKey: 'storage_class', width: '14%', hideBelow: BP.lg },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(pv: PersistentVolumeInfo)}
  {@const h = health(pv)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="Persistent Volumes"
  viewId="pvs"
  itemLabel="persistent volumes"
  subtitle="cluster-scoped"
  source={pvList}
  {columns}
  rowKey={(pv) => pv.name}
  filterFields={['name', 'status', 'claim', 'storage_class']}
  {health}
  segmentLabels={{ failed: 'Failed', warning: 'Released', progressing: 'Pending', healthy: 'Bound', neutral: 'Available' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  resourceType="pv"
  namespaced={false}
/>
