<script lang="ts">
  import ResourceListView from '../ResourceListView.svelte';
  import { roles, type RoleInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<RoleInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '18%' },
    { id: 'rules', label: 'Rules', sortKey: 'rules', numeric: true, width: '80px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ResourceListView
  title="Roles"
  itemLabel="roles"
  viewId="roles"
  store={roles}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  resourceType="role"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
