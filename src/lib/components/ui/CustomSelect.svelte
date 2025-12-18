<script lang="ts">
  import { onMount } from 'svelte';

  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    options: Option[];
    value: string;
    placeholder?: string;
    disabled?: boolean;
    onchange?: (value: string) => void;
  }

  let { options, value = $bindable(), placeholder = 'Select...', disabled = false, onchange }: Props = $props();

  let isOpen = $state(false);
  let dropdownRef: HTMLDivElement;
  let buttonRef: HTMLButtonElement;

  function toggle() {
    if (!disabled) {
      isOpen = !isOpen;
    }
  }

  function select(optionValue: string) {
    value = optionValue;
    isOpen = false;
    onchange?.(optionValue);
  }

  function handleClickOutside(e: MouseEvent) {
    if (dropdownRef && !dropdownRef.contains(e.target as Node)) {
      isOpen = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      isOpen = false;
      buttonRef?.focus();
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    return () => document.removeEventListener('click', handleClickOutside);
  });

  const selectedLabel = $derived(
    options.find(o => o.value === value)?.label || placeholder
  );
</script>

<div class="relative" bind:this={dropdownRef}>
  <button
    bind:this={buttonRef}
    type="button"
    onclick={toggle}
    {disabled}
    class="w-full flex items-center justify-between px-3 py-2 bg-bg-primary border border-border-subtle rounded-lg text-sm text-left focus:outline-none focus:border-accent-primary transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    class:text-text-primary={value}
    class:text-text-muted={!value}
  >
    <span class="truncate">{selectedLabel}</span>
    <svg
      class="w-4 h-4 text-text-muted flex-shrink-0 ml-2 transition-transform"
      class:rotate-180={isOpen}
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if isOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="absolute z-50 w-full mt-1 bg-bg-secondary border border-border-subtle rounded-lg shadow-xl overflow-hidden"
      onkeydown={handleKeydown}
    >
      <div class="max-h-48 overflow-y-auto">
        {#if placeholder}
          <button
            type="button"
            onclick={() => select('')}
            class="w-full px-3 py-2 text-sm text-left text-text-muted hover:bg-bg-tertiary transition-colors"
          >
            {placeholder}
          </button>
        {/if}
        {#each options as option}
          <button
            type="button"
            onclick={() => select(option.value)}
            class="w-full px-3 py-2 text-sm text-left transition-colors flex items-center justify-between {option.value === value ? 'bg-accent-primary/10 text-accent-primary' : 'text-text-primary hover:bg-bg-tertiary'}"
          >
            <span class="truncate">{option.label}</span>
            {#if option.value === value}
              <svg class="w-4 h-4 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
