<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import { BP, type Column } from '../ui/data-table';
  import { poll } from '../../utils/poll';
  import {
    ingresses,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadIngresses,
    resourceState,
    type IngressInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('ingresses');

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadIngresses(ns));
  });

  const columns: Column<IngressInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    { id: 'class', label: 'Class', sortKey: 'class', width: '12%', hideBelow: BP.lg },
    // An ingress without hosts matches every host.
    { id: 'hosts', label: 'Hosts', sortKey: (i) => i.hosts.join(','), mono: true, width: '26%', cell: hostsCell, title: (i) => i.hosts.join(', ') || '*' },
    { id: 'address', label: 'Address', sortKey: 'address', mono: true, width: '16%', hideBelow: BP.wide },
    { id: 'ports', label: 'Ports', sortKey: 'ports', mono: true, width: '80px', hideBelow: BP.lg },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet hostsCell(i: IngressInfo)}{i.hosts.length ? i.hosts.join(', ') : '*'}{/snippet}

<ListView
  title="Ingresses"
  viewId="ingresses"
  itemLabel="ingresses"
  items={$ingresses}
  loadState={$loadState}
  {columns}
  rowKey={(i) => `${i.namespace}/${i.name}`}
  filterFields={['name', 'namespace', 'class', 'hosts', 'address']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="ingress"
  onRefresh={() => loadIngresses($selectedNamespace)}
/>
