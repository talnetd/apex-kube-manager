<script lang="ts">
  import ListView from '../../ui/ListView.svelte';
  import { roleBindings, type RoleBindingInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<RoleBindingInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '16%' },
    { id: 'role_ref', label: 'Role', sortKey: 'role_ref', width: '22%', mono: true },
    { id: 'subjects', label: 'Subjects', sortKey: 'subject_count', width: '28%', hideBelow: 900 },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ListView
  title="Role Bindings"
  itemLabel="role bindings"
  viewId="rolebindings"
  source={roleBindings}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  resourceType="rolebinding"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
