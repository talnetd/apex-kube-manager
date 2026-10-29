<script lang="ts">
  import ResourceListView from '../ResourceListView.svelte';
  import { ingressClasses, type IngressClassInfo } from '../../../stores/resources';
  import type { Column } from '../../ui/data-table';

  const columns: Column<IngressClassInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true, cell: nameCell },
    { id: 'controller', label: 'Controller', sortKey: 'controller', width: '34%', mono: true },
    { id: 'parameters', label: 'Parameters', sortKey: 'parameters', width: '24%', hideBelow: 900 },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet nameCell(r: IngressClassInfo)}{r.name}{#if r.is_default}<span class="deftag">default</span>{/if}{/snippet}

<ResourceListView
  title="Ingress Classes"
  itemLabel="ingress classes"
  viewId="ingressclasses"
  store={ingressClasses}
  {columns}
  rowKey={(r) => r.name}
  resourceType="ingressclass"
  defaultSort={{ id: 'name', dir: 'asc' }}
/>
