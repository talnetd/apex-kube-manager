<script lang="ts">
  // Square icon-only button (spec 5.6). `label` is required: it is the
  // aria-label and the tooltip (with the key hint appended when given).
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { BUTTON_VARIANTS, type ButtonVariant, type ButtonSize } from './Button.svelte';
  import { tooltip } from '../../actions/tooltip';
  import Spinner from './Spinner.svelte';

  let {
    label,
    shortcut,
    variant = 'ghost',
    size = 'md',
    active = false,
    loading = false,
    disabled = false,
    ref = $bindable(),
    class: className = '',
    children,
    ...rest
  }: {
    label: string;
    /** Key hint shown in the tooltip, e.g. "L". */
    shortcut?: string;
    variant?: ButtonVariant;
    size?: ButtonSize;
    /** Toggled-on state (e.g. an open panel): bg-active + text-primary. */
    active?: boolean;
    loading?: boolean;
    disabled?: boolean;
    ref?: HTMLButtonElement;
    class?: string;
    children: Snippet;
  } & Omit<HTMLButtonAttributes, 'type' | 'disabled' | 'class' | 'children'> = $props();

  const SIZES: Record<ButtonSize, string> = { sm: 'w-6 h-6', md: 'w-7 h-7', lg: 'w-8 h-8' };
  const tip = $derived(shortcut ? `${label} · ${shortcut}` : label);
</script>

<button
  bind:this={ref}
  type="button"
  aria-label={label}
  aria-pressed={active ? 'true' : undefined}
  disabled={disabled || loading}
  aria-busy={loading ? 'true' : undefined}
  use:tooltip={tip}
  class="inline-flex items-center justify-center shrink-0 rounded-md transition-colors duration-100 ease-out disabled:opacity-45 disabled:cursor-not-allowed disabled:pointer-events-none {SIZES[size]} {BUTTON_VARIANTS[variant]} {active ? 'bg-active! text-primary!' : ''} {className}"
  {...rest}
>
  {#if loading}<Spinner />{:else}<span class="inline-flex w-4 h-4 items-center justify-center" aria-hidden="true">{@render children()}</span>{/if}
</button>
