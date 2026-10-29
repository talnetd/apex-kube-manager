<script lang="ts">
  import ListView from '../../ui/ListView.svelte';
  import { roles, type RoleInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<RoleInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '18%' },
    { id: 'rules', label: 'Rules', sortKey: 'rules', numeric: true, width: '80px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ListView
  title="Roles"
  itemLabel="roles"
  viewId="roles"
  source={roles}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  resourceType="role"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
