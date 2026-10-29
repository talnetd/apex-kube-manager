<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, type Column } from '../ui/data-table';
  import { verdictHealth } from '../../utils/status-words';
  import { poll } from '../../utils/poll';
  import {
    daemonsets,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadDaemonSets,
    resourceState,
    type DaemonSetInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('daemonsets');
  const health = verdictHealth<DaemonSetInfo>();

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadDaemonSets(ns));
  });

  const columns: Column<DaemonSetInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    {
      id: 'status',
      label: 'Status',
      width: '18%',
      sortKey: (d) => health(d).rank,
      cell: statusCell,
      title: (d) => `${d.verdict.reason} (${d.verdict.ruleId})`,
    },
    { id: 'desired', label: 'Desired', sortKey: 'desired', numeric: true, width: '76px' },
    { id: 'current', label: 'Current', sortKey: 'current', numeric: true, width: '76px', hideBelow: BP.lg },
    { id: 'ready', label: 'Ready', sortKey: 'ready', numeric: true, width: '64px' },
    { id: 'up_to_date', label: 'Up-to-date', sortKey: 'up_to_date', numeric: true, width: '96px', hideBelow: BP.lg },
    { id: 'available', label: 'Available', sortKey: 'available', numeric: true, width: '84px', hideBelow: BP.wide },
    { id: 'node_selector', label: 'Node selector', sortKey: 'node_selector', mono: true, width: '16%', hideBelow: BP.lg },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(d: DaemonSetInfo)}
  {@const h = health(d)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="DaemonSets"
  viewId="daemonsets"
  itemLabel="daemonsets"
  items={$daemonsets}
  loadState={$loadState}
  {columns}
  rowKey={(d) => `${d.namespace}/${d.name}`}
  filterFields={['name', 'namespace', 'node_selector', (d) => d.verdict.reason]}
  {health}
  segmentLabels={{ failed: 'Failing', warning: 'Degraded', progressing: 'Rolling out', healthy: 'Ready', neutral: 'No nodes' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  resourceType="daemonset"
  onRefresh={() => loadDaemonSets($selectedNamespace)}
/>
