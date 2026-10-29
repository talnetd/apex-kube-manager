<script lang="ts">
  // Create stays a header action. Edit is a row action (it opens the rule
  // builder, nothing changes until Update). Delete moved off the row into the
  // editor (NetworkPolicyCreator) behind a ConfirmDialog (CLAUDE.md,
  // Architecture Note 7).
  import { invoke } from '@tauri-apps/api/core';
  import ListView from '../ui/ListView.svelte';
  import Button from '../ui/Button.svelte';
  import IconButton from '../ui/IconButton.svelte';
  import NetworkPolicyCreator from '../ui/NetworkPolicyCreator.svelte';
  import { BP, type Column } from '../ui/data-table';
  import { poll } from '../../utils/poll';
  import { toast } from '../../stores/toast';
  import {
    networkPolicies,
    selectedNamespace,
    currentContext,
    refreshTrigger,
    loadNetworkPolicies,
    resourceState,
    type NetworkPolicyInfo,
  } from '../../stores/kubernetes';

  const loadState = resourceState('networkpolicies');

  $effect(() => {
    if (!$currentContext) return;
    void $refreshTrigger;
    const ns = $selectedNamespace;
    return poll(() => loadNetworkPolicies(ns));
  });

  const columns: Column<NetworkPolicyInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '16%', hideBelow: BP.md },
    // An empty selector selects every pod in the namespace.
    { id: 'pod_selector', label: 'Pod selector', sortKey: 'pod_selector', mono: true, width: '28%', cell: selectorCell, title: (n) => n.pod_selector || '<all>' },
    { id: 'policy_types', label: 'Types', sortKey: (n) => n.policy_types.join(','), width: '140px', hideBelow: BP.wide },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];

  let showCreator = $state(false);
  // Shape comes from get_networkpolicy_detail; NetworkPolicyCreator reads it.
  let editingPolicy = $state<any>(null);

  async function edit(np: NetworkPolicyInfo) {
    try {
      editingPolicy = await invoke('get_networkpolicy_detail', {
        contextName: $currentContext,
        namespace: np.namespace,
        name: np.name,
      });
      showCreator = true;
    } catch (e) {
      toast(`Couldn't load ${np.name}: ${e}`, { health: 'failed' });
    }
  }

  function closeCreator() {
    showCreator = false;
    editingPolicy = null;
  }
</script>

{#snippet selectorCell(n: NetworkPolicyInfo)}{n.pod_selector || '<all>'}{/snippet}

{#snippet actions()}
  <Button onclick={() => (showCreator = true)}>Create</Button>
{/snippet}

{#snippet rowActions(n: NetworkPolicyInfo)}
  <IconButton label="Edit" shortcut="E" size="sm" onclick={() => edit(n)}>
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M10.5 2.5l3 3L6 13H3v-3z" />
    </svg>
  </IconButton>
{/snippet}

<ListView
  title="Network Policies"
  viewId="networkpolicies"
  itemLabel="network policies"
  items={$networkPolicies}
  loadState={$loadState}
  {columns}
  rowKey={(n) => `${n.namespace}/${n.name}`}
  filterFields={['name', 'namespace', 'pod_selector', 'policy_types']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="networkpolicy"
  onRefresh={() => loadNetworkPolicies($selectedNamespace)}
  keyboard={!showCreator}
  {actions}
  {rowActions}
  shortcuts={{
    e: edit,
    d: (n) => {
      edit(n);
      toast('Delete from the edit dialog');
    },
  }}
/>

<NetworkPolicyCreator show={showCreator} editData={editingPolicy} onclose={closeCreator} />

