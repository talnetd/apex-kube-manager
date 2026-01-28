<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { currentContext, selectedNamespace, namespaces, loadSecrets } from '../../stores/kubernetes';
  import CustomSelect from './CustomSelect.svelte';

  // Edit data structure matching backend SecretDetail
  interface EditData {
    name: string;
    namespace: string;
    secret_type: string;
    data: Record<string, string>;
  }

  interface Props {
    show: boolean;
    onclose: () => void;
    editData?: EditData | null;
  }

  let { show, onclose, editData = null }: Props = $props();

  // Form state
  let name = $state('');
  let editNamespace = $state<string | null>(null);
  let createNamespace = $state<string>('');
  let secretType = $state<string>('Opaque');
  let dataEntries = $state<{key: string, value: string, revealed: boolean}[]>([]);

  // UI state
  let isSaving = $state(false);
  let error = $state<string | null>(null);

  // Track if we're in edit mode
  let isEditMode = $derived(editData !== null);

  // Namespace options for dropdown
  const namespaceOptions = $derived(
    $namespaces.map(ns => ({ value: ns, label: ns }))
  );

  // Secret type options
  const secretTypeOptions = [
    { value: 'Opaque', label: 'Opaque (Generic)' },
    { value: 'kubernetes.io/basic-auth', label: 'Basic Auth' },
    { value: 'kubernetes.io/ssh-auth', label: 'SSH Auth' },
    { value: 'kubernetes.io/tls', label: 'TLS' },
    { value: 'kubernetes.io/dockerconfigjson', label: 'Docker Config' },
    { value: 'kubernetes.io/service-account-token', label: 'Service Account Token' },
  ];

  // Populate form from edit data
  function populateFromEditData(data: EditData) {
    name = data.name;
    editNamespace = data.namespace;
    secretType = data.secret_type || 'Opaque';

    // Convert data map to entries array
    // Note: data from backend is already decoded (we use get_secret_data which decodes)
    dataEntries = Object.entries(data.data || {}).map(([key, value]) => ({
      key,
      value,
      revealed: false
    }));
  }

  // Watch for editData changes and populate form
  $effect(() => {
    if (show && editData) {
      populateFromEditData(editData);
    } else if (show && !editData) {
      reset();
      createNamespace = $selectedNamespace || $namespaces[0] || 'default';
    }
  });

  function reset() {
    name = '';
    editNamespace = null;
    createNamespace = $selectedNamespace || 'default';
    secretType = 'Opaque';
    dataEntries = [];
    error = null;
  }

  function addEntry() {
    dataEntries = [...dataEntries, { key: '', value: '', revealed: false }];
  }

  function removeEntry(index: number) {
    dataEntries = dataEntries.filter((_, i) => i !== index);
  }

  function toggleReveal(index: number) {
    dataEntries[index].revealed = !dataEntries[index].revealed;
    dataEntries = [...dataEntries];
  }

  function entriesToMap(): Record<string, string> {
    const map: Record<string, string> = {};
    for (const entry of dataEntries) {
      if (entry.key.trim()) {
        map[entry.key.trim()] = entry.value;
      }
    }
    return map;
  }

  async function handleSave() {
    if (!name.trim()) {
      error = 'Name is required';
      return;
    }

    // Validate no empty keys
    const hasEmptyKey = dataEntries.some(e => e.value && !e.key.trim());
    if (hasEmptyKey) {
      error = 'All entries must have a key';
      return;
    }

    const namespace = isEditMode ? (editNamespace || 'default') : (createNamespace || 'default');

    isSaving = true;
    error = null;

    try {
      const data = entriesToMap();

      if (isEditMode) {
        await invoke('update_secret', {
          contextName: $currentContext,
          namespace,
          name: name.trim(),
          secretType,
          data,
        });
      } else {
        await invoke('create_secret', {
          contextName: $currentContext,
          namespace,
          name: name.trim(),
          secretType,
          data,
        });
      }

      await loadSecrets(namespace);
      reset();
      onclose();
    } catch (e) {
      error = String(e);
    } finally {
      isSaving = false;
    }
  }

  function handleClose() {
    reset();
    onclose();
  }
</script>

{#if show}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onclick={handleClose}>
    <div
      class="bg-bg-secondary rounded-lg border border-border-subtle w-[600px] max-h-[85vh] flex flex-col"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-border-subtle">
        <h2 class="text-lg font-semibold text-text-primary">{isEditMode ? 'Edit' : 'Create'} Secret</h2>
        <button
          onclick={handleClose}
          class="p-1 hover:bg-bg-tertiary rounded transition-colors"
        >
          <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4">
        <div class="space-y-4">
          <!-- Name -->
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-1">Name *</label>
            {#if isEditMode}
              <div class="text-sm text-text-primary bg-bg-tertiary border border-border-subtle rounded px-3 py-2 opacity-60">
                {name}
              </div>
              <p class="text-xs text-text-muted mt-1">Name cannot be changed when editing</p>
            {:else}
              <input
                type="text"
                bind:value={name}
                placeholder="my-secret"
                class="w-full bg-bg-tertiary border border-border-subtle rounded px-3 py-2 text-text-primary text-sm focus:outline-none focus:border-accent-primary"
              />
            {/if}
          </div>

          <!-- Namespace -->
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-1">Namespace *</label>
            {#if isEditMode}
              <div class="text-sm text-text-primary bg-bg-tertiary border border-border-subtle rounded px-3 py-2 opacity-60">
                {editNamespace || 'default'}
              </div>
              <p class="text-xs text-text-muted mt-1">Namespace cannot be changed when editing</p>
            {:else}
              <CustomSelect
                value={createNamespace}
                options={namespaceOptions}
                placeholder="Select namespace..."
                onchange={(v) => createNamespace = v}
              />
            {/if}
          </div>

          <!-- Secret Type -->
          <div>
            <label class="block text-sm font-medium text-text-secondary mb-1">Type</label>
            <CustomSelect
              value={secretType}
              options={secretTypeOptions}
              onchange={(v) => secretType = v}
            />
          </div>

          <!-- Data Entries -->
          <div>
            <div class="flex items-center justify-between mb-2">
              <label class="block text-sm font-medium text-text-secondary">Data</label>
              <button
                onclick={addEntry}
                class="text-xs px-2 py-1 bg-accent-primary/10 text-accent-primary rounded hover:bg-accent-primary/20 transition-colors"
              >
                + Add Entry
              </button>
            </div>

            {#if dataEntries.length === 0}
              <div class="text-center py-8 bg-bg-tertiary rounded-lg border border-border-subtle border-dashed">
                <svg class="w-8 h-8 text-text-muted mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                <p class="text-sm text-text-muted">No secret data</p>
                <p class="text-xs text-text-muted mt-1">Click "Add Entry" to add key-value pairs</p>
              </div>
            {:else}
              <div class="space-y-2">
                {#each dataEntries as entry, i}
                  <div class="flex gap-2 items-start bg-bg-tertiary rounded-lg p-3">
                    <div class="flex-1 space-y-2">
                      <input
                        type="text"
                        bind:value={entry.key}
                        placeholder="Key (e.g., password)"
                        class="w-full bg-bg-secondary border border-border-subtle rounded px-3 py-1.5 text-text-primary text-sm focus:outline-none focus:border-accent-primary"
                      />
                      <div class="relative">
                        {#if entry.revealed}
                          <textarea
                            bind:value={entry.value}
                            placeholder="Value"
                            rows="2"
                            class="w-full bg-bg-secondary border border-border-subtle rounded px-3 py-1.5 text-text-primary text-sm focus:outline-none focus:border-accent-primary font-mono resize-none"
                          ></textarea>
                        {:else}
                          <input
                            type="password"
                            bind:value={entry.value}
                            placeholder="Value (hidden)"
                            class="w-full bg-bg-secondary border border-border-subtle rounded px-3 py-1.5 text-text-primary text-sm focus:outline-none focus:border-accent-primary font-mono"
                          />
                        {/if}
                      </div>
                    </div>
                    <div class="flex flex-col gap-1">
                      <button
                        onclick={() => toggleReveal(i)}
                        class="p-1.5 rounded hover:bg-bg-secondary transition-colors text-text-muted hover:text-text-primary"
                        title={entry.revealed ? 'Hide value' : 'Show value'}
                      >
                        {#if entry.revealed}
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                          </svg>
                        {:else}
                          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                          </svg>
                        {/if}
                      </button>
                      <button
                        onclick={() => removeEntry(i)}
                        class="p-1.5 rounded hover:bg-accent-error/10 transition-colors text-text-muted hover:text-accent-error"
                        title="Remove entry"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
            {/if}

            <p class="text-xs text-text-muted mt-2">
              Values are stored securely. Base64 encoding is handled automatically.
            </p>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between p-4 border-t border-border-subtle">
        {#if error}
          <p class="text-sm text-accent-error">{error}</p>
        {:else}
          <div></div>
        {/if}
        <div class="flex gap-3">
          <button
            onclick={handleClose}
            class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={handleSave}
            disabled={isSaving}
            class="px-4 py-2 text-sm bg-accent-primary text-bg-primary rounded hover:bg-accent-primary/80 transition-colors disabled:opacity-50"
          >
            {isSaving ? (isEditMode ? 'Updating...' : 'Creating...') : (isEditMode ? 'Update Secret' : 'Create Secret')}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
