<script lang="ts">
  // Empty / filtered / error / forbidden / unavailable states (spec 5.5).
  // Sits 48px below the table header, not vertically centred, so the eye
  // stays near the header.
  import type { Snippet } from 'svelte';
  import StatusIcon from './StatusIcon.svelte';

  type Variant = 'empty' | 'filtered' | 'error' | 'forbidden' | 'unavailable';

  let {
    variant = 'empty',
    title,
    hint,
    detail,
    actions,
    icon,
  }: {
    variant?: Variant;
    title: string;
    hint?: string;
    /** Mono block, e.g. the raw error text. */
    detail?: string;
    actions?: Snippet;
    /** Custom 24px icon; defaults per variant. */
    icon?: Snippet;
  } = $props();
</script>

<div
  class="mx-auto max-w-[360px] pt-12 pb-8 px-4 flex flex-col items-center text-center"
  role={variant === 'error' ? 'alert' : 'status'}
>
  <div class="mb-3 {variant === 'error' ? 'text-danger-solid' : 'text-muted'}" aria-hidden="true">
    {#if icon}
      {@render icon()}
    {:else if variant === 'error'}
      <StatusIcon health="failed" size={24} colored={false} />
    {:else if variant === 'forbidden'}
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <rect x="5" y="11" width="14" height="9" rx="2" />
        <path d="M8 11V8a4 4 0 0 1 8 0v3" />
      </svg>
    {:else if variant === 'filtered'}
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="6.5" />
        <path d="M20 20l-4.2-4.2" />
      </svg>
    {:else if variant === 'unavailable'}
      <StatusIcon health="unknown" size={24} colored={false} />
    {:else}
      <StatusIcon health="neutral" size={24} colored={false} />
    {/if}
  </div>
  <p class="text-sm font-medium text-primary">{title}</p>
  {#if hint}<p class="mt-1 text-sm text-muted">{hint}</p>{/if}
  {#if detail}
    <pre class="mt-3 w-full max-h-40 overflow-auto text-left font-mono text-xs text-secondary bg-subtle border border-line rounded-md p-2 whitespace-pre-wrap break-words">{detail}</pre>
  {/if}
  {#if actions}
    <div class="mt-3 flex items-center justify-center gap-2">{@render actions()}</div>
  {/if}
</div>
