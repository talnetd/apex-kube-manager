<script lang="ts">
  import ResourceListView from '../ResourceListView.svelte';
  import IconButton from '../../ui/IconButton.svelte';
  import { crds, customResources, selectedCrdName, type CrdInfo } from '../../../stores/resources';
  import { currentView } from '../../../stores/navigation';
  import type { Column } from '../../ui/data-table';

  const columns: Column<CrdInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'kind', label: 'Kind', sortKey: 'kind', width: '18%' },
    { id: 'group', label: 'Group', sortKey: 'group', width: '22%', mono: true, hideBelow: 900 },
    { id: 'scope', label: 'Scope', sortKey: 'scope', width: '110px' },
    { id: 'version', label: 'Version', sortKey: 'version', width: '90px', mono: true },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];

  function browse(c: CrdInfo) {
    if (c.name !== $selectedCrdName) customResources.reset();
    selectedCrdName.set(c.name);
    currentView.set('customresources');
  }
</script>

{#snippet rowActions(c: CrdInfo)}
  <IconButton label="Browse {c.kind} objects" shortcut="B" size="sm" onclick={() => browse(c)}>
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"
      ><path d="M3 4h10M3 8h10M3 12h6" /></svg
    >
  </IconButton>
{/snippet}

<ResourceListView
  title="Definitions"
  subtitle="CustomResourceDefinitions"
  itemLabel="definitions"
  viewId="crds"
  store={crds}
  {columns}
  rowKey={(r) => r.name}
  resourceType="crd"
  defaultSort={{ id: 'name', dir: 'asc' }}
  {rowActions}
  emptyHint="This cluster has no CustomResourceDefinitions installed."
/>
