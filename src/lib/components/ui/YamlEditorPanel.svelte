<script lang="ts">
  import { guard, detailReadOnly } from '../../stores/detail';
  import { invoke } from '@tauri-apps/api/core';
  import YamlEditor from './YamlEditor.svelte';
  import Button from './Button.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import { toast } from '../../stores/toast';

  interface Props {
    yaml: string;
    context: string;
    resourceType: string;
    /** The object, so the apply dialog's prod banner names it. */
    name?: string;
    namespace?: string;
    isDeleted?: boolean;
    onRefresh: () => Promise<void>;
    onApplySuccess?: () => Promise<void>;
  }

  let { yaml, context, resourceType, name, namespace, isDeleted = false, onRefresh, onApplySuccess }: Props = $props();

  // Editing state
  let isEditing = $state<boolean>(false);
  let editedYaml = $state<string>('');
  let isSaving = $state<boolean>(false);
  let saveMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);

  // Turning read-only mid-edit keeps the unsaved text; only Apply is off
  // (saveYaml guards too) until the user switches back.
  function startEditing() {
    if (!guard('edit')) return;
    editedYaml = yaml;
    isEditing = true;
    saveMessage = null;
  }

  /** Cancel asks first when there are unsaved changes (like Refresh). */
  function requestCancel() {
    if (editedYaml !== yaml) confirmDiscard = 'cancel';
    else cancelEditing();
  }

  function cancelEditing() {
    isEditing = false;
    editedYaml = '';
    saveMessage = null;
  }

  function handleYamlChange(newContent: string) {
    editedYaml = newContent;
  }

  // Discarding unsaved edits asks first (ConfirmDialog, no native confirm()).
  let confirmDiscard = $state<'cancel' | 'refresh' | null>(null);
  let confirmApply = $state(false);

  async function handleRefresh() {
    if (isEditing && editedYaml !== yaml) {
      confirmDiscard = 'refresh';
      return;
    }
    if (isEditing) cancelEditing();
    await onRefresh();
  }

  async function discard() {
    const thenRefresh = confirmDiscard === 'refresh';
    cancelEditing();
    if (thenRefresh) await onRefresh();
  }

  function saveYaml() {
    if (!guard('apply')) return;
    if (isDeleted || !editedYaml) return;
    confirmApply = true;
  }

  // Runs from the ConfirmDialog (prod guard applies); throws only when the
  // apply itself fails, so the dialog shows that error. A failed refresh
  // afterwards just toasts: the change is already applied.
  async function doApply() {
    if (!guard('apply')) throw new Error('Switch back to this context to apply');
    isSaving = true;
    saveMessage = null;
    try {
      await invoke<string>('apply_yaml', { contextName: context, yamlContent: editedYaml });
    } finally {
      isSaving = false;
    }
    try {
      // Fetch the new manifest first, so the editor never flashes the old one.
      await onRefresh();
      if (onApplySuccess) await onApplySuccess();
    } catch (e) {
      toast(`Applied, but reloading failed: ${e}`, { health: 'warning' });
    }
    isEditing = false;
    editedYaml = '';
  }

  async function copyYaml() {
    try {
      await navigator.clipboard.writeText(isEditing ? editedYaml : yaml);
      toast('Copied YAML');
    } catch {
      toast("Couldn't copy to clipboard", { health: 'failed' });
    }
  }

  // Reset edit state when yaml prop changes significantly (e.g., switching resources)
  $effect(() => {
    // When the source yaml changes and we're not editing, clear any messages
    if (!isEditing && yaml) {
      saveMessage = null;
    }
  });

  // Export a method to reset state (useful when parent wants to reset)
  export function resetState() {
    isEditing = false;
    editedYaml = '';
    saveMessage = null;
  }
</script>

<div class="h-full flex flex-col">
  <div class="ybar">
    <span class="text-sm text-secondary">
      {resourceType} manifest
      {#if isEditing}<span class="editing">editing</span>{/if}
    </span>
    <div class="flex items-center gap-1.5">
      {#if isEditing}
        <Button size="sm" variant="ghost" disabled={isSaving} onclick={requestCancel}>Cancel</Button>
        <Button
          size="sm"
          variant="primary"
          loading={isSaving}
          disabled={isDeleted || $detailReadOnly}
          title={$detailReadOnly ? 'Read-only: the app moved to another context' : undefined}
          onclick={saveYaml}>Apply…</Button
        >
      {:else}
        <Button size="sm" disabled={!yaml} onclick={copyYaml}>Copy</Button>
        <Button
          size="sm"
          disabled={isDeleted || !yaml || $detailReadOnly}
          title={$detailReadOnly ? 'Read-only: the app moved to another context' : undefined}
          onclick={startEditing}>Edit</Button
        >
        <Button size="sm" variant="ghost" disabled={isDeleted} onclick={handleRefresh}>Refresh</Button>
      {/if}
    </div>
  </div>
  {#if saveMessage}
    <div class="msg" class:err={saveMessage.type === 'error'} role={saveMessage.type === 'error' ? 'alert' : 'status'}>{saveMessage.text}</div>
  {/if}
  <div class="flex-1 overflow-hidden">
    {#if yaml || isEditing}
      <YamlEditor content={isEditing ? editedYaml : yaml} readonly={!isEditing} onchange={handleYamlChange} />
    {:else}
      <p class="text-muted text-sm px-5 py-8">Loading YAML…</p>
    {/if}
  </div>
</div>

{#if confirmApply}
  <ConfirmDialog
    open={true}
    title={`Apply changes to this ${resourceType.toLowerCase()}?`}
    resource={{ kind: resourceType, name: name ?? resourceType, namespace, context }}
    confirmLabel="Apply"
    body="kubectl apply with the edited manifest. Fields you removed may be pruned by the API server."
    successMessage="Applied"
    onConfirm={doApply}
    onClose={() => (confirmApply = false)}
  />
{:else if confirmDiscard}
  <ConfirmDialog
    open={true}
    title="Discard your edits?"
    resource={{ kind: resourceType, name: name ?? resourceType, namespace, context }}
    confirmLabel={confirmDiscard === 'refresh' ? 'Discard and refresh' : 'Discard'}
    tone="danger"
    prodEscalation={false}
    body={confirmDiscard === 'refresh' ? 'Refreshing loads the live manifest and drops what you typed.' : 'What you typed is dropped and the live manifest comes back.'}
    onConfirm={discard}
    onClose={() => (confirmDiscard = null)}
  />
{/if}

<style>
  .ybar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 8px;
    padding: 8px 20px;
    border-bottom: 1px solid var(--ui-line);
  }
  .editing {
    margin-left: 8px;
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 0 5px;
    border: 1px solid color-mix(in srgb, var(--ui-warning) 55%, transparent);
    border-radius: 4px;
    color: var(--ui-warning);
  }
  .msg {
    padding: 8px 20px;
    font-size: 13px;
    color: var(--ui-secondary);
    border-bottom: 1px solid var(--ui-line);
    overflow-wrap: anywhere;
  }
  .msg.err {
    color: var(--ui-danger);
  }
  @media (max-width: 639.98px) {
    .ybar,
    .msg {
      padding-left: 12px;
      padding-right: 12px;
    }
  }
</style>
