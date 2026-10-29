<script lang="ts">
  import { onDestroy } from 'svelte';
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, type Column } from '../ui/data-table';
  import { verdictHealth } from '../../utils/status-words';
  import { parseAge } from '../../utils/sort';
  import {
    selectedNamespace,
    currentContext,
    refreshTrigger,
    startJobWatch,
    stopJobWatch,
    jobList,
    type JobInfo,
  } from '../../stores/kubernetes';

  const health = verdictHealth<JobInfo>();

  // Starts the watch on mount, and again on namespace, context or refresh.
  $effect(() => {
    const ctx = $currentContext;
    void $refreshTrigger;
    if (!ctx) return;
    startJobWatch($selectedNamespace);
  });
  onDestroy(() => stopJobWatch());

  const columns: Column<JobInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    {
      id: 'status',
      label: 'Status',
      width: '18%',
      sortKey: (j) => health(j).rank,
      cell: statusCell,
      title: (j) => `${j.verdict.reason} (${j.verdict.ruleId})`,
    },
    { id: 'completions', label: 'Completions', sortKey: 'completions', numeric: true, width: '104px' },
    { id: 'duration', label: 'Duration', sortKey: (j) => (j.duration ? parseAge(j.duration) : null), numeric: true, width: '84px', hideBelow: BP.wide },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(j: JobInfo)}
  {@const h = health(j)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="Jobs"
  viewId="jobs"
  itemLabel="jobs"
  source={jobList}
  autoload={false}
  {columns}
  rowKey={(j) => `${j.namespace}/${j.name}`}
  filterFields={['name', 'namespace', 'status', (j) => j.verdict.reason]}
  {health}
  segmentLabels={{ failed: 'Failed', warning: 'Retrying', progressing: 'Running', healthy: 'Complete', neutral: 'Suspended' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  resourceType="job"
  onRefresh={() => startJobWatch($selectedNamespace)}
/>
