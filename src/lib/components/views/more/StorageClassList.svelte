<script lang="ts">
  import ListView from '../../ui/ListView.svelte';
  import { storageClasses, type StorageClassInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<StorageClassInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true, cell: nameCell },
    { id: 'provisioner', label: 'Provisioner', sortKey: 'provisioner', width: '26%', mono: true },
    { id: 'reclaim_policy', label: 'Reclaim', sortKey: 'reclaim_policy', width: '96px' },
    { id: 'binding_mode', label: 'Binding mode', sortKey: 'binding_mode', width: '150px', hideBelow: 900 },
    { id: 'allow_expansion', label: 'Expansion', sortKey: (r) => (r.allow_expansion ? 1 : 0), width: '96px', cell: expansionCell },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet nameCell(r: StorageClassInfo)}{r.name}{#if r.is_default}<span class="deftag">default</span>{/if}{/snippet}
{#snippet expansionCell(r: StorageClassInfo)}{r.allow_expansion ? 'Allowed' : '–'}{/snippet}

<ListView
  title="Storage Classes"
  itemLabel="storage classes"
  viewId="storageclasses"
  source={storageClasses}
  {columns}
  rowKey={(r) => r.name}
  resourceType="storageclass"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
