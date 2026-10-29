<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import { BP, type Column } from '../ui/data-table';
  import {
    configMapList,
    type ConfigMapInfo,
  } from '../../stores/kubernetes';

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
  source={configMapList}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  filterFields={['name', 'namespace']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="configmap"
/>
