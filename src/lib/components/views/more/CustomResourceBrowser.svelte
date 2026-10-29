<script lang="ts">
  // Generic custom resource browser (spec 12.3): pick a CRD, list its
  // objects through the dynamic API (name, namespace, age), open one for a
  // YAML-first detail window.
  import { onMount } from 'svelte';
  import ResourceListView from '../ResourceListView.svelte';
  import CustomSelect from '../../ui/CustomSelect.svelte';
  import { crds, customResources, selectedCrdName, type CustomResourceInfo } from '../../../stores/resources';
  import { currentContext } from '../../../stores/kubernetes';
  import { crdTarget, encodeCrType } from '../../../utils/resources';
  import type { Column } from '../../ui/data-table';

  const crdItems = crds.items;
  const crdState = crds.state;

  // The picker needs the CRD list; load it once here if Definitions wasn't opened.
  onMount(() => {
    if ($crdState.status === 'idle') crds.load();
  });
  $effect(() => {
    if ($currentContext && $crdState.status === 'idle') crds.load();
  });

  const crd = $derived($crdItems.find((c) => c.name === $selectedCrdName) ?? null);
  const target = $derived(crd ? crdTarget(crd) : null);
  // A stable object per target, so the list only reloads when the pick changes.
  const extra = $derived(target ? { target } : undefined);

  const targetKey = $derived(target ? encodeCrType(target) : null);

  // A pick from another cluster doesn't exist here: drop it.
  $effect(() => {
    if ($selectedCrdName && $crdState.status === 'ready' && !crd) pick(null);
  });

  // Objects of the previous type must not show under the new one, so the
  // list clears before the new type loads.
  function pick(name: string | null) {
    if (name === $selectedCrdName) return;
    customResources.reset();
    selectedCrdName.set(name);
  }

  const options = $derived(
    [...$crdItems]
      .sort((a, b) => a.kind.localeCompare(b.kind) || a.group.localeCompare(b.group))
      .map((c) => ({ value: c.name, label: `${c.kind} · ${c.group}` }))
  );

  const columns = $derived<Column<CustomResourceInfo>[]>([
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    ...(target?.namespaced ? [{ id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '20%' } as Column<CustomResourceInfo>] : []),
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ]);
</script>

{#snippet picker()}
  <div class="w-[280px]">
    <CustomSelect
      value={$selectedCrdName ?? ''}
      {options}
      placeholder={$crdState.status === 'ready' ? 'Pick a resource type…' : 'Loading definitions…'}
      disabled={!options.length}
      onchange={(v) => pick(v || null)}
    />
  </div>
{/snippet}

{#key targetKey ?? 'none'}
  <ResourceListView
    title={crd ? crd.kind : 'Custom Resources'}
    subtitle={crd ? `${crd.group}/${crd.version}` : undefined}
    itemLabel={crd ? crd.plural : 'objects'}
    viewId="customresources"
    store={customResources}
    {columns}
    rowKey={(r) => `${r.namespace}/${r.name}`}
    resourceType={target ? encodeCrType(target) : 'crd'}
    defaultSort={{ id: 'name', dir: 'asc' }}
    {extra}
    enabled={!!target}
    actions={picker}
    emptyHint={target ? undefined : 'Pick a resource type above to list its objects.'}
  />
{/key}
