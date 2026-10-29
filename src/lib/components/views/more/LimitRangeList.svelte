<script lang="ts">
  import ListView from '../../ui/ListView.svelte';
  import { limitRanges, type LimitRangeInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<LimitRangeInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '18%' },
    { id: 'types', label: 'Applies to', sortKey: 'types', width: '24%' },
    { id: 'limits', label: 'Limits', sortKey: 'limits', numeric: true, width: '80px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ListView
  title="Limit Ranges"
  itemLabel="limit ranges"
  viewId="limitranges"
  source={limitRanges}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  resourceType="limitrange"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
