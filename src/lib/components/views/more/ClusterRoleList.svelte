<script lang="ts">
  import ResourceListView from '../ResourceListView.svelte';
  import { clusterRoles, type ClusterRoleInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<ClusterRoleInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'rules', label: 'Rules', sortKey: 'rules', numeric: true, width: '80px' },
    { id: 'aggregated', label: 'Aggregated', sortKey: (r) => (r.aggregated ? 1 : 0), width: '110px', cell: aggregatedCell },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet aggregatedCell(r: ClusterRoleInfo)}{r.aggregated ? 'Yes' : '–'}{/snippet}

<ResourceListView
  title="Cluster Roles"
  itemLabel="cluster roles"
  viewId="clusterroles"
  store={clusterRoles}
  {columns}
  rowKey={(r) => r.name}
  resourceType="clusterrole"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
