<script lang="ts">
  import ResourceListView from '../ResourceListView.svelte';
  import { clusterRoleBindings, type ClusterRoleBindingInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<ClusterRoleBindingInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'role_ref', label: 'Role', sortKey: 'role_ref', width: '26%', mono: true },
    { id: 'subjects', label: 'Subjects', sortKey: 'subject_count', width: '32%', hideBelow: 900 },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ResourceListView
  title="Cluster Role Bindings"
  itemLabel="cluster role bindings"
  viewId="clusterrolebindings"
  store={clusterRoleBindings}
  {columns}
  rowKey={(r) => r.name}
  resourceType="clusterrolebinding"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
