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
  import { toast } from '../../stores/toast';
  import { get } from 'svelte/store';
  import {
    networkPolicyList,
    currentContext,
    type NetworkPolicyInfo,
  } from '../../stores/kubernetes';

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

  // The context the editor targets: the one the row was loaded from, fixed
  // when the editor opens. NetworkPolicyCreator closes itself if it changes.
  let editContext = $state('');

  function create() {
    editingPolicy = null;
    editContext = $currentContext;
    showCreator = true;
  }

  async function edit(np: NetworkPolicyInfo) {
    // Rows clear on a context switch; this is the backstop for a click that
    // lands mid-switch.
    const ctx = get(networkPolicyList.context);
    if (!ctx || ctx !== $currentContext) return void toast('Still loading this cluster. Try again in a moment.');
    try {
      const detail = await invoke('get_networkpolicy_detail', {
        contextName: ctx,
        namespace: np.namespace,
        name: np.name,
      });
      if ($currentContext !== ctx) return;
      editingPolicy = detail;
      editContext = ctx;
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
  <Button onclick={create}>Create</Button>
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
  source={networkPolicyList}
  {columns}
  rowKey={(n) => `${n.namespace}/${n.name}`}
  filterFields={['name', 'namespace', 'pod_selector', 'policy_types']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="networkpolicy"
  keyboard={!showCreator}
  {actions}
  {rowActions}
  shortcuts={{ e: edit }}
  deleteHint="Delete from Edit (E)"
/>

<NetworkPolicyCreator show={showCreator} editData={editingPolicy} context={editContext} onclose={closeCreator} />

