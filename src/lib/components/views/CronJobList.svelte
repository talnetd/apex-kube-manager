<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, type Column } from '../ui/data-table';
  import { cronJobHealth } from '../../utils/status-words';
  import { poll } from '../../utils/poll';
  import { parseAge } from '../../utils/sort';
  import {
    cronjobs,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadCronJobs,
    resourceState,
    type CronJobInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('cronjobs');

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadCronJobs(ns));
  });

  const columns: Column<CronJobInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    { id: 'schedule', label: 'Schedule', sortKey: 'schedule', mono: true, width: '16%', hideBelow: BP.md },
    { id: 'status', label: 'Status', width: '16%', sortKey: (c) => cronJobHealth(c).rank, cell: statusCell },
    { id: 'active', label: 'Active', sortKey: 'active', numeric: true, width: '64px', hideBelow: BP.lg },
    { id: 'last_schedule', label: 'Last run', sortKey: (c) => (c.last_schedule ? parseAge(c.last_schedule) : null), numeric: true, width: '84px', hideBelow: BP.wide },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(c: CronJobInfo)}
  {@const h = cronJobHealth(c)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="CronJobs"
  viewId="cronjobs"
  itemLabel="cronjobs"
  items={$cronjobs}
  loadState={$loadState}
  {columns}
  rowKey={(c) => `${c.namespace}/${c.name}`}
  filterFields={['name', 'namespace', 'schedule']}
  health={cronJobHealth}
  segmentLabels={{ progressing: 'Active', healthy: 'Scheduled', neutral: 'Suspended' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  resourceType="cronjob"
  onRefresh={() => loadCronJobs($selectedNamespace)}
/>
