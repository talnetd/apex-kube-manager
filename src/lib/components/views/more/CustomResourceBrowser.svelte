<script lang="ts">
  // Generic custom resource browser (spec 12.3): pick a CRD, list its
  // objects through the dynamic API (name, namespace, age), open one for a
  // YAML-first detail window.
  import { onMount } from 'svelte';
  import ListView from '../../ui/ListView.svelte';
  import CustomSelect from '../../ui/CustomSelect.svelte';
  import Button from '../../ui/Button.svelte';
  import { crds, customResources, selectedCrdName, type CustomResourceInfo } from '../../../stores/resources';
  import { currentContext, refreshTrigger } from '../../../stores/kubernetes';
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
  // Refresh (r / header button) retries a failed definitions load too.
  let lastTrigger = $refreshTrigger;
  $effect(() => {
    const t = $refreshTrigger;
    if (t !== lastTrigger) {
      lastTrigger = t;
      if ($crdState.status === 'error' || $crdState.status === 'forbidden') crds.load();
    }
  });
  const crdsFailed = $derived($crdState.status === 'error' || $crdState.status === 'forbidden');
  const pickerHint = $derived(
    $crdState.status === 'forbidden'
      ? 'No access to CustomResourceDefinitions'
      : $crdState.status === 'error'
        ? "Couldn't load definitions"
        : $crdState.status === 'ready'
          ? $crdItems.length
            ? 'Pick a resource type…'
            : 'No CRDs in this cluster'
          : 'Loading definitions…'
  );

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
  <div class="w-[280px]" title={crdsFailed ? $crdState.error : undefined}>
    <CustomSelect
      value={$selectedCrdName ?? ''}
      {options}
      placeholder={pickerHint}
      disabled={!options.length}
      onchange={(v) => pick(v || null)}
    />
  </div>
  {#if crdsFailed}
    <Button size="sm" onclick={() => crds.load()}>Retry</Button>
  {/if}
{/snippet}

{#key targetKey ?? 'none'}
  <ListView
    title={crd ? crd.kind : 'Custom Resources'}
    subtitle={crd ? `${crd.group}/${crd.version}` : undefined}
    itemLabel={crd ? crd.plural : 'objects'}
    viewId="customresources"
    source={customResources}
    {columns}
    rowKey={(r) => `${r.namespace}/${r.name}`}
    resourceType={target ? encodeCrType(target) : 'crd'}
    defaultSort={{ id: 'name', dir: 'asc' }}
    {extra}
    namespaced={target ? target.namespaced : true}
    enabled={!!target}
    actions={picker}
    emptyHint={target ? undefined : crdsFailed ? `${pickerHint}. ${$crdState.error ?? ''}` : 'Pick a resource type above to list its objects.'}
  />
{/key}
