<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import { type Column } from '../ui/data-table';
  import { namespaceHealth } from '../../utils/status-words';
  import { poll } from '../../utils/poll';
  import {
    namespacesInfo,
    currentContext,
    refreshTrigger,
    loadNamespacesInfo,
    resourceState,
    type NamespaceInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('namespaces');
  const health = (n: NamespaceInfo) => namespaceHealth(n.status);

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    return poll(() => loadNamespacesInfo());
  });

  const columns: Column<NamespaceInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'status', label: 'Status', width: '160px', sortKey: (n) => health(n).rank, cell: statusCell },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(n: NamespaceInfo)}
  {@const h = health(n)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="Namespaces"
  viewId="namespaces"
  itemLabel="namespaces"
  subtitle="cluster-scoped"
  items={$namespacesInfo}
  loadState={$loadState}
  {columns}
  rowKey={(n) => n.name}
  filterFields={['name', 'status']}
  {health}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="namespace"
  namespaced={false}
  onRefresh={() => loadNamespacesInfo()}
/>
