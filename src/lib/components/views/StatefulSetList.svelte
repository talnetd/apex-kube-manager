<script lang="ts">
  // Scale and restart live in StatefulSetDetail (CLAUDE.md, Architecture
  // Note 7); `s` opens it so the old shortcut still lands somewhere useful.
  import { invoke } from '@tauri-apps/api/core';
  import { get } from 'svelte/store';
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { BP, HEALTH_SORT_ID, type Column } from '../ui/data-table';
  import { verdictHealth } from '../../utils/status-words';
  import { toast } from '../../stores/toast';
  import {
    statefulSetList,
    currentContext,
    type StatefulSetInfo,
  } from '../../stores/kubernetes';

  const health = verdictHealth<StatefulSetInfo>();

  const columns: Column<StatefulSetInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    {
      id: 'status',
      label: 'Status',
      width: '18%',
      sortKey: (s) => health(s).rank,
      cell: statusCell,
      title: (s) => `${s.verdict.reason} (${s.verdict.ruleId})`,
    },
    { id: 'ready', label: 'Ready', sortKey: 'ready', numeric: true, width: '72px' },
    { id: 'replicas', label: 'Replicas', sortKey: 'replicas', numeric: true, width: '84px', hideBelow: BP.lg },
    { id: 'service_name', label: 'Service', sortKey: 'service_name', mono: true, width: '16%', hideBelow: BP.wide },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];

  function openDetail(s: StatefulSetInfo) {
    invoke('open_resource_detail', {
      resourceType: 'statefulset',
      name: s.name,
      namespace: s.namespace,
      // The context the rows came from (ListView refuses rows from another).
      context: get(statefulSetList.context) || $currentContext,
    }).catch((e) => console.error('Failed to open statefulset detail:', e));
  }

  function scale(s: StatefulSetInfo) {
    openDetail(s);
    toast('Scale from the detail window');
  }
</script>

{#snippet statusCell(s: StatefulSetInfo)}
  {@const h = health(s)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="StatefulSets"
  viewId="statefulsets"
  itemLabel="statefulsets"
  source={statefulSetList}
  {columns}
  rowKey={(s) => `${s.namespace}/${s.name}`}
  filterFields={['name', 'namespace', 'service_name', (s) => s.verdict.reason]}
  {health}
  segmentLabels={{ failed: 'Failing', warning: 'Degraded', progressing: 'Rolling out', healthy: 'Ready', neutral: 'Scaled to 0' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  onOpen={openDetail}
  shortcuts={{ s: scale }}
/>
