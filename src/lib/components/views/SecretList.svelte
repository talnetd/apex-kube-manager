<script lang="ts">
  // Create stays a header action. Edit is a row action (it opens the editor,
  // nothing changes until Update). Delete moved off the row into the editor
  // (SecretCreator) behind a ConfirmDialog (CLAUDE.md, Architecture Note 7).
  import { invoke } from '@tauri-apps/api/core';
  import ListView from '../ui/ListView.svelte';
  import Button from '../ui/Button.svelte';
  import IconButton from '../ui/IconButton.svelte';
  import SecretCreator from '../ui/SecretCreator.svelte';
  import { BP, type Column } from '../ui/data-table';
  import { toast } from '../../stores/toast';
  import { get } from 'svelte/store';
  import {
    secretList,
    currentContext,
    type SecretInfo,
  } from '../../stores/kubernetes';

  const columns: Column<SecretInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '16%', hideBelow: BP.md },
    { id: 'secret_type', label: 'Type', sortKey: 'secret_type', mono: true, width: '26%', hideBelow: BP.wide },
    { id: 'data_count', label: 'Keys', sortKey: 'data_count', numeric: true, width: '64px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];

  let showCreator = $state(false);
  let editingSecret = $state<{ name: string; namespace: string; secret_type: string; data: Record<string, string> } | null>(null);

  // The context the editor targets: the one the row was loaded from, fixed
  // when the editor opens. SecretCreator closes itself if it changes.
  let editContext = $state('');

  function create() {
    editingSecret = null;
    editContext = $currentContext;
    showCreator = true;
  }

  async function edit(sec: SecretInfo) {
    // Rows clear on a context switch; this is the backstop for a click that
    // lands mid-switch.
    const ctx = get(secretList.context);
    if (!ctx || ctx !== $currentContext) return void toast('Still loading this cluster. Try again in a moment.');
    try {
      const data = await invoke<Record<string, string>>('get_secret_data', {
        contextName: ctx,
        namespace: sec.namespace,
        name: sec.name,
      });
      if ($currentContext !== ctx) return;
      editingSecret = { name: sec.name, namespace: sec.namespace, secret_type: sec.secret_type, data };
      editContext = ctx;
      showCreator = true;
    } catch (e) {
      toast(`Couldn't load ${sec.name}: ${e}`, { health: 'failed' });
    }
  }

  function closeCreator() {
    showCreator = false;
    editingSecret = null;
  }
</script>

{#snippet actions()}
  <Button onclick={create}>Create</Button>
{/snippet}

{#snippet rowActions(s: SecretInfo)}
  <IconButton label="Edit" shortcut="E" size="sm" onclick={() => edit(s)}>
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M10.5 2.5l3 3L6 13H3v-3z" />
    </svg>
  </IconButton>
{/snippet}

<ListView
  title="Secrets"
  viewId="secrets"
  itemLabel="secrets"
  source={secretList}
  {columns}
  rowKey={(s) => `${s.namespace}/${s.name}`}
  filterFields={['name', 'namespace', 'secret_type']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="secret"
  keyboard={!showCreator}
  {actions}
  {rowActions}
  shortcuts={{ e: edit }}
  deleteHint="Delete from Edit (E)"
/>

<SecretCreator show={showCreator} editData={editingSecret} context={editContext} onclose={closeCreator} />

