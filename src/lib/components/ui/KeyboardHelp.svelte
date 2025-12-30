<script lang="ts">
  import { showKeyboardHelp, keyboardShortcuts } from '../../stores/keyboard';

  function close() {
    showKeyboardHelp.set(false);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }

  const navigationShortcuts = keyboardShortcuts.filter(s => s.category === 'navigation');
  const actionShortcuts = keyboardShortcuts.filter(s => s.category === 'actions');
  const globalShortcuts = keyboardShortcuts.filter(s => s.category === 'global');
</script>

<svelte:window onkeydown={handleKeydown} />

{#if $showKeyboardHelp}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" role="dialog" aria-modal="true" onclick={close}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div
      class="bg-bg-card border border-border-subtle rounded-lg p-6 max-w-lg w-full mx-4 shadow-xl"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-lg font-semibold text-text-primary">Keyboard Shortcuts</h2>
        <button
          onclick={close}
          class="p-1 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
          title="Close"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="space-y-6">
        <!-- Navigation -->
        <div>
          <h3 class="text-xs text-text-muted uppercase tracking-wide mb-3">Navigation</h3>
          <div class="space-y-2">
            {#each navigationShortcuts as shortcut}
              <div class="flex items-center justify-between">
                <span class="text-text-secondary text-sm">{shortcut.description}</span>
                <kbd class="px-2 py-1 bg-bg-tertiary border border-border-subtle rounded text-xs text-text-primary font-mono">
                  {shortcut.key}
                </kbd>
              </div>
            {/each}
          </div>
        </div>

        <!-- Actions -->
        <div>
          <h3 class="text-xs text-text-muted uppercase tracking-wide mb-3">Actions</h3>
          <div class="space-y-2">
            {#each actionShortcuts as shortcut}
              <div class="flex items-center justify-between">
                <span class="text-text-secondary text-sm">{shortcut.description}</span>
                <kbd class="px-2 py-1 bg-bg-tertiary border border-border-subtle rounded text-xs text-text-primary font-mono">
                  {shortcut.key}
                </kbd>
              </div>
            {/each}
          </div>
        </div>

        <!-- Global -->
        <div>
          <h3 class="text-xs text-text-muted uppercase tracking-wide mb-3">Global</h3>
          <div class="space-y-2">
            {#each globalShortcuts as shortcut}
              <div class="flex items-center justify-between">
                <span class="text-text-secondary text-sm">{shortcut.description}</span>
                <kbd class="px-2 py-1 bg-bg-tertiary border border-border-subtle rounded text-xs text-text-primary font-mono">
                  {shortcut.key}
                </kbd>
              </div>
            {/each}
          </div>
        </div>
      </div>

      <div class="mt-6 pt-4 border-t border-border-subtle">
        <p class="text-xs text-text-muted text-center">
          Press <kbd class="px-1.5 py-0.5 bg-bg-tertiary border border-border-subtle rounded text-xs font-mono">?</kbd> anytime to show this help
        </p>
      </div>
    </div>
  </div>
{/if}
