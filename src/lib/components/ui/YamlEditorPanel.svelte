<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import YamlEditor from './YamlEditor.svelte';

  interface Props {
    yaml: string;
    context: string;
    resourceType: string;
    isDeleted?: boolean;
    onRefresh: () => Promise<void>;
    onApplySuccess?: () => Promise<void>;
  }

  let { yaml, context, resourceType, isDeleted = false, onRefresh, onApplySuccess }: Props = $props();

  // Editing state
  let isEditing = $state<boolean>(false);
  let editedYaml = $state<string>('');
  let isSaving = $state<boolean>(false);
  let saveMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);

  function startEditing() {
    editedYaml = yaml;
    isEditing = true;
    saveMessage = null;
  }

  function cancelEditing() {
    isEditing = false;
    editedYaml = '';
    saveMessage = null;
  }

  function handleYamlChange(newContent: string) {
    editedYaml = newContent;
  }

  async function handleRefresh() {
    if (isEditing) {
      // If editing, ask for confirmation
      if (!confirm('You have unsaved changes. Discard and refresh?')) {
        return;
      }
      cancelEditing();
    }
    await onRefresh();
  }

  async function saveYaml() {
    if (isDeleted || !editedYaml) return;

    try {
      isSaving = true;
      saveMessage = null;
      const result = await invoke<string>('apply_yaml', {
        contextName: context,
        yamlContent: editedYaml
      });
      saveMessage = { type: 'success', text: result || 'Applied successfully' };
      isEditing = false;
      editedYaml = '';
      // Reload the YAML and detail
      await onRefresh();
      if (onApplySuccess) {
        await onApplySuccess();
      }
    } catch (e) {
      saveMessage = { type: 'error', text: String(e) };
    } finally {
      isSaving = false;
    }
  }

  async function copyYaml() {
    await navigator.clipboard.writeText(isEditing ? editedYaml : yaml);
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
  <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
    <div class="flex items-center gap-2">
      <span class="text-sm text-text-muted">{resourceType} YAML Manifest</span>
      {#if isEditing}
        <span class="text-xs px-2 py-0.5 rounded bg-accent-warning/20 text-accent-warning">Editing</span>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      {#if isEditing}
        <button
          onclick={cancelEditing}
          disabled={isSaving}
          class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          onclick={saveYaml}
          disabled={isSaving || isDeleted}
          class="text-xs px-3 py-1 bg-accent-primary text-white rounded hover:bg-accent-primary/90 transition-colors disabled:opacity-50 flex items-center gap-1"
        >
          {#if isSaving}
            <svg class="w-3 h-3 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            Applying...
          {:else}
            Apply
          {/if}
        </button>
      {:else}
        <button
          onclick={copyYaml}
          disabled={!yaml}
          class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
        >
          Copy
        </button>
        <button
          onclick={startEditing}
          disabled={isDeleted || !yaml}
          class="text-xs px-3 py-1 bg-accent-primary/10 text-accent-primary rounded hover:bg-accent-primary/20 transition-colors disabled:opacity-50"
        >
          Edit
        </button>
        <button
          onclick={handleRefresh}
          disabled={isDeleted}
          class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50"
        >
          Refresh
        </button>
      {/if}
    </div>
  </div>
  {#if saveMessage}
    <div class="px-4 py-2 text-sm {saveMessage.type === 'success' ? 'bg-accent-success/10 text-accent-success' : 'bg-accent-error/10 text-accent-error'}">
      {saveMessage.text}
    </div>
  {/if}
  <div class="flex-1 overflow-hidden">
    {#if yaml || isEditing}
      <YamlEditor
        content={isEditing ? editedYaml : yaml}
        readonly={!isEditing}
        onchange={handleYamlChange}
      />
    {:else}
      <div class="flex items-center justify-center h-full text-text-muted">
        Loading YAML...
      </div>
    {/if}
  </div>
</div>
