<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import { BP, type Column } from '../ui/data-table';
  import { poll } from '../../utils/poll';
  import {
    configmaps,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadConfigMaps,
    resourceState,
    type ConfigMapInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('configmaps');

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadConfigMaps(ns));
  });

  const columns: Column<ConfigMapInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '20%', hideBelow: BP.md },
    { id: 'data_count', label: 'Keys', sortKey: 'data_count', numeric: true, width: '72px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ListView
  title="ConfigMaps"
  viewId="configmaps"
  itemLabel="configmaps"
  items={$configmaps}
  loadState={$loadState}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  filterFields={['name', 'namespace']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="configmap"
  onRefresh={() => loadConfigMaps($selectedNamespace)}
/>
