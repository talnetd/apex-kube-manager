<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import { BP, type Column } from '../ui/data-table';
  import { poll } from '../../utils/poll';
  import {
    hpas,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadHPAs,
    resourceState,
    type HPAInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('hpas');

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadHPAs(ns));
  });

  const columns: Column<HPAInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    { id: 'reference', label: 'Reference', sortKey: 'reference', mono: true, width: '20%', hideBelow: BP.wide },
    { id: 'targets', label: 'Targets', sortKey: 'targets', mono: true, width: '16%' },
    { id: 'min_max', label: 'Min / max', sortKey: 'max_pods', numeric: true, width: '88px', hideBelow: BP.lg, cell: minMaxCell },
    { id: 'replicas', label: 'Replicas', sortKey: 'replicas', numeric: true, width: '80px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet minMaxCell(h: HPAInfo)}{h.min_pods} / {h.max_pods}{/snippet}

<ListView
  title="HPAs"
  viewId="hpas"
  itemLabel="autoscalers"
  items={$hpas}
  loadState={$loadState}
  {columns}
  rowKey={(h) => `${h.namespace}/${h.name}`}
  filterFields={['name', 'namespace', 'reference']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="hpa"
  onRefresh={() => loadHPAs($selectedNamespace)}
/>
