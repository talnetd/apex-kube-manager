<script lang="ts">
  // Scale and restart live in DeploymentDetail (CLAUDE.md, Architecture
  // Note 7); `s` opens it so the old shortcut still lands somewhere useful.
  import { onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/core';
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, type Column } from '../ui/data-table';
  import { verdictHealth } from '../../utils/status-words';
  import { toast } from '../../stores/toast';
  import {
    selectedNamespace,
    currentContext,
    refreshTrigger,
    startDeploymentWatch,
    stopDeploymentWatch,
    deploymentList,
    type DeploymentInfo,
  } from '../../stores/kubernetes';

  const health = verdictHealth<DeploymentInfo>();

  // Starts the watch on mount, and again on namespace, context or refresh.
  $effect(() => {
    const ctx = $currentContext;
    void $refreshTrigger;
    if (!ctx) return;
    startDeploymentWatch($selectedNamespace);
  });
  onDestroy(() => stopDeploymentWatch());

  const columns: Column<DeploymentInfo>[] = [
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
    { id: 'ready', label: 'Ready', sortKey: 'ready', numeric: true, width: '72px' },
    { id: 'up_to_date', label: 'Up-to-date', sortKey: 'up_to_date', numeric: true, width: '96px', hideBelow: BP.lg },
    { id: 'available', label: 'Available', sortKey: 'available', numeric: true, width: '88px', hideBelow: BP.wide },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];

  function openDetail(d: DeploymentInfo) {
    invoke('open_resource_detail', {
      resourceType: 'deployment',
      name: d.name,
      namespace: d.namespace,
      // The context the rows came from (ListView refuses rows from another).
      context: get(deploymentList.context) || $currentContext,
    }).catch((e) => console.error('Failed to open deployment detail:', e));
  }

  function scale(d: DeploymentInfo) {
    openDetail(d);
    toast('Scale from the detail window');
  }
</script>

{#snippet statusCell(d: DeploymentInfo)}
  {@const h = health(d)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="Deployments"
  viewId="deployments"
  itemLabel="deployments"
  source={deploymentList}
  autoload={false}
  {columns}
  rowKey={(d) => `${d.namespace}/${d.name}`}
  filterFields={['name', 'namespace', (d) => d.verdict.reason]}
  {health}
  segmentLabels={{ failed: 'Failing', warning: 'Degraded', progressing: 'Rolling out', healthy: 'Ready', neutral: 'Scaled to 0' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  onOpen={openDetail}
  onRefresh={() => startDeploymentWatch($selectedNamespace)}
  shortcuts={{ s: scale }}
/>
