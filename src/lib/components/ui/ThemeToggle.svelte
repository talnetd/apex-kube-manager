<script lang="ts">
  import { theme, resolvedTheme, setTheme, type Theme } from "../../stores/theme";

  let isOpen = $state(false);

  function selectTheme(newTheme: Theme) {
    setTheme(newTheme);
    isOpen = false;
  }

  function toggleDropdown() {
    isOpen = !isOpen;
  }

  // Close dropdown when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest(".theme-toggle-container")) {
      isOpen = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      document.addEventListener("click", handleClickOutside);
      return () => document.removeEventListener("click", handleClickOutside);
    }
  });

// Icon components
const SunIcon = () => `
  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M12 18a6 6 0 100-12 6 6 0 000 12z"
    />
  </svg>
`;

const MoonIcon = () => `
  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
    />
  </svg>
`;

const MonitorIcon = () => `
  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M9.75 17.25L9 21h6l-.75-3.75M3 13.5V6a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 6v7.5A2.25 2.25 0 0118.75 15.75H5.25A2.25 2.25 0 013 13.5z"
    />
  </svg>
`;

const CheckIcon = () => `
  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M5 13l4 4L19 7"
    />
  </svg>
`;
</script>

<div class="theme-toggle-container relative">
  <button
    onclick={toggleDropdown}
    class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-bg-secondary hover:bg-bg-tertiary transition-colors text-text-secondary hover:text-text-primary border border-border-subtle"
  >
    {#if $resolvedTheme === "light"}
      {@render SunIcon()}
    {:else if $resolvedTheme === "dark"}
      {@render MoonIcon()}
    {:else}
      {@render MonitorIcon()}
    {/if}

    <span>Change theme</span>

    <span class:rotate-180={isOpen} class="transition-transform">
      {@render ChevronDownIcon()}
    </span>
  </button>

  {#if isOpen}
    <div
      class="absolute right-0 mt-2 w-40 bg-bg-secondary border border-border-subtle rounded-lg shadow-xl overflow-hidden z-50"
    >
      <button
        onclick={() => selectTheme("light")}
        class="w-full flex items-center justify-between px-4 py-2.5 text-sm text-text-primary hover:bg-bg-tertiary transition-colors"
      >
        <div class="flex items-center gap-2">
          {@render SunIcon()}
          <span>Light</span>
        </div>
        {#if $theme === "light"}
          {@render CheckIcon()}
        {/if}
      </button>

      <button
        onclick={() => selectTheme("dark")}
        class="w-full flex items-center justify-between px-4 py-2.5 text-sm text-text-primary hover:bg-bg-tertiary transition-colors"
      >
        <div class="flex items-center gap-2">
          {@render MoonIcon()}
          <span>Dark</span>
        </div>
        {#if $theme === "dark"}
          {@render CheckIcon()}
        {/if}
      </button>

      <button
        onclick={() => selectTheme("system")}
        class="w-full flex items-center justify-between px-4 py-2.5 text-sm text-text-primary hover:bg-bg-tertiary transition-colors"
      >
        <div class="flex items-center gap-2">
          {@render MonitorIcon()}
          <span>System</span>
        </div>
        {#if $theme === "system"}
          {@render CheckIcon()}
        {/if}
      </button>
    </div>
  {/if}
</div>

<style>
  /* Ensure dropdown appears above other content */
  .theme-toggle-container {
    position: relative;
    z-index: 10;
  }
</style>

