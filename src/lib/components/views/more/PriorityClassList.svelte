<script lang="ts">
  import ResourceListView from '../ResourceListView.svelte';
  import { priorityClasses, type PriorityClassInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<PriorityClassInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true, cell: nameCell },
    { id: 'value', label: 'Value', sortKey: 'value', numeric: true, width: '120px' },
    { id: 'preemption_policy', label: 'Preemption', sortKey: 'preemption_policy', width: '180px', hideBelow: 900 },
    { id: 'description', label: 'Description', sortKey: 'description', width: '32%', hideBelow: 1100 },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet nameCell(r: PriorityClassInfo)}{r.name}{#if r.global_default}<span class="deftag">default</span>{/if}{/snippet}

<ResourceListView
  title="Priority Classes"
  itemLabel="priority classes"
  viewId="priorityclasses"
  store={priorityClasses}
  {columns}
  rowKey={(r) => r.name}
  resourceType="priorityclass"
  defaultSort={{ id: 'value', dir: 'desc' }}
/>
