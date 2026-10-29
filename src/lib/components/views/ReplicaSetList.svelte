<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, type Column } from '../ui/data-table';
  import { verdictHealth } from '../../utils/status-words';
  import {
    replicaSetList,
    type ReplicaSetInfo,
  } from '../../stores/kubernetes';

  const health = verdictHealth<ReplicaSetInfo>();

  const columns: Column<ReplicaSetInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    {
      id: 'status',
      label: 'Status',
      width: '18%',
      sortKey: (r) => health(r).rank,
      cell: statusCell,
      title: (r) => `${r.verdict.reason} (${r.verdict.ruleId})`,
    },
    { id: 'desired', label: 'Desired', sortKey: 'desired', numeric: true, width: '76px' },
    { id: 'current', label: 'Current', sortKey: 'current', numeric: true, width: '76px', hideBelow: BP.wide },
    { id: 'ready', label: 'Ready', sortKey: 'ready', numeric: true, width: '64px' },
    { id: 'owner', label: 'Owner', sortKey: 'owner', mono: true, width: '18%', hideBelow: BP.lg },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(r: ReplicaSetInfo)}
  {@const h = health(r)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="ReplicaSets"
  viewId="replicasets"
  itemLabel="replicasets"
  source={replicaSetList}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  filterFields={['name', 'namespace', 'owner', (r) => r.verdict.reason]}
  {health}
  segmentLabels={{ failed: 'Failing', warning: 'Degraded', progressing: 'Rolling out', healthy: 'Ready', neutral: 'Scaled to 0' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  resourceType="replicaset"
/>
