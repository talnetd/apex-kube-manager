<script lang="ts" module>
  export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'danger-ghost';
  export type ButtonSize = 'sm' | 'md' | 'lg';

  export const BUTTON_VARIANTS: Record<ButtonVariant, string> = {
    primary: 'btn-bright bg-accent text-on-accent',
    secondary: 'bg-surface border border-line text-primary hover:bg-hover hover:border-line-strong active:bg-active',
    ghost: 'bg-transparent text-secondary hover:bg-hover hover:text-primary active:bg-active',
    danger: 'btn-bright bg-danger-button text-white',
    'danger-ghost': 'bg-transparent text-danger hover:bg-danger-tint active:bg-danger-tint',
  };

  export const BUTTON_SIZES: Record<ButtonSize, string> = {
    sm: 'h-6 text-xs px-2 gap-1.5',
    md: 'h-7 text-sm px-2.5 gap-1.5',
    lg: 'h-8 text-sm px-3 gap-2',
  };
</script>

<script lang="ts">
  // Button (spec 5.6). Disabled = opacity 0.45, no hover. Loading keeps the
  // label and swaps the icon for a spinner, so the width doesn't jump.
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import Kbd from './Kbd.svelte';
  import Spinner from './Spinner.svelte';

  let {
    variant = 'secondary',
    size = 'md',
    icon,
    kbd,
    loading = false,
    disabled = false,
    type = 'button',
    ref = $bindable(),
    class: className = '',
    children,
    ...rest
  }: {
    variant?: ButtonVariant;
    size?: ButtonSize;
    icon?: Snippet;
    kbd?: string;
    loading?: boolean;
    disabled?: boolean;
    type?: 'button' | 'submit' | 'reset';
    ref?: HTMLButtonElement;
    class?: string;
    children?: Snippet;
  } & Omit<HTMLButtonAttributes, 'type' | 'disabled' | 'class' | 'children'> = $props();

  const inert = $derived(disabled || loading);
</script>

<button
  bind:this={ref}
  {type}
  disabled={inert}
  aria-busy={loading ? 'true' : undefined}
  class="inline-flex items-center justify-center shrink-0 rounded-md font-medium whitespace-nowrap select-none transition-colors duration-100 ease-out disabled:opacity-45 disabled:cursor-not-allowed disabled:pointer-events-none {BUTTON_SIZES[size]} {BUTTON_VARIANTS[variant]} {className}"
  {...rest}
>
  {#if loading}
    <Spinner size={size === 'sm' ? 12 : 14} />
  {:else if icon}
    <span class="inline-flex shrink-0 w-4 h-4 items-center justify-center" aria-hidden="true">{@render icon()}</span>
  {/if}
  {#if children}<span class="truncate">{@render children()}</span>{/if}
  {#if kbd}<Kbd keys={kbd} class="ml-0.5 {variant === 'primary' || variant === 'danger' ? 'kbd-on-fill' : ''}" />{/if}
</button>

<style>
  .btn-bright:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  .btn-bright:active:not(:disabled) {
    filter: brightness(0.95);
  }
  :global(.kbd-on-fill) {
    background: rgb(255 255 255 / 0.16) !important;
    border-color: rgb(255 255 255 / 0.28) !important;
    color: rgb(255 255 255 / 0.9) !important;
  }
</style>
