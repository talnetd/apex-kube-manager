<script lang="ts">
  // Toast stack (spec 5.7): bottom-right, above the 24px status bar slot,
  // max 3. Mount once per window.
  import { fly } from 'svelte/transition';
  import { cubicOut } from 'svelte/easing';
  import { toasts, dismissToast } from '../../stores/toast';
  import StatusIcon from './StatusIcon.svelte';
</script>

<div
  class="fixed right-4 bottom-10 z-[900] flex flex-col items-end gap-2 pointer-events-none"
  aria-live="polite"
  aria-atomic="false"
>
  {#each $toasts as t (t.id)}
    <div
      class="pointer-events-auto flex items-center gap-2 max-w-[360px] bg-surface text-primary shadow-popover rounded-lg pl-3 pr-1.5 py-1.5 text-sm"
      role="status"
      in:fly={{ y: 8, duration: 150, easing: cubicOut }}
      out:fly={{ y: 8, duration: 150 }}
    >
      {#if t.icon === 'shield'}
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linejoin="round" class="text-warning-solid shrink-0" aria-hidden="true">
          <path d="M8 1.75l5.25 2v4.1c0 3.1-2.2 5.4-5.25 6.4-3.05-1-5.25-3.3-5.25-6.4v-4.1z" />
        </svg>
      {:else if t.health}
        <StatusIcon health={t.health} size={14} />
      {/if}
      <span class="min-w-0 flex-1 break-words">{t.message}</span>
      <button
        type="button"
        class="shrink-0 w-6 h-6 inline-flex items-center justify-center rounded-md text-muted hover:bg-hover hover:text-primary"
        aria-label="Dismiss"
        onclick={() => dismissToast(t.id)}
      >
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
          <path d="M4 4l8 8M12 4l-8 8" />
        </svg>
      </button>
    </div>
  {/each}
</div>
