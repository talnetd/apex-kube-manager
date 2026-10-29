<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import { BP, type Column } from '../ui/data-table';
  import { poll } from '../../utils/poll';
  import {
    serviceAccounts,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadServiceAccounts,
    resourceState,
    type ServiceAccountInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('serviceaccounts');

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadServiceAccounts(ns));
  });

  const columns: Column<ServiceAccountInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '20%', hideBelow: BP.md },
    { id: 'secrets', label: 'Secrets', sortKey: 'secrets', numeric: true, width: '80px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ListView
  title="Service Accounts"
  viewId="serviceaccounts"
  itemLabel="service accounts"
  items={$serviceAccounts}
  loadState={$loadState}
  {columns}
  rowKey={(s) => `${s.namespace}/${s.name}`}
  filterFields={['name', 'namespace']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="serviceaccount"
  onRefresh={() => loadServiceAccounts($selectedNamespace)}
/>
