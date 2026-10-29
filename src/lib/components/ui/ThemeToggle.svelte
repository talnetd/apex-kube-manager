<script lang="ts">
  import { theme, setTheme, type Theme } from "../../stores/theme";

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
    if (!isOpen) return;
    document.addEventListener("click", handleClickOutside);
    return () => document.removeEventListener("click", handleClickOutside);
  });
  // Icon components
  const SunIcon = `
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
        d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M12 18a6 6 0 100-12 6 6 0 000 12z" />
    </svg>
  `;

  const MoonIcon = `
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
        d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
    </svg>
  `;

  const MonitorIcon = `
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
        d="M9.75 17.25L9 21h6l-.75-3.75M3 13.5V6a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 6v7.5A2.25 2.25 0 0118.75 15.75H5.25A2.25 2.25 0 013 13.5z" />
    </svg>
  `;


  const CheckIcon = `
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
    </svg>
  `;

  const ChevronDownIcon = `
    <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </svg>
  `;

</script>

<div class="theme-toggle-container relative">
  <button
    type="button"
    onclick={toggleDropdown}
    title="Theme"
    aria-label="Change theme"
    aria-haspopup="menu"
    aria-expanded={isOpen}
    class="w-7 h-7 inline-grid place-items-center rounded-md text-secondary hover:bg-hover hover:text-primary"
  >
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"
      ><circle cx="8" cy="8" r="5.5" /><path d="M8 2.5a5.5 5.5 0 0 0 0 11z" fill="currentColor" /></svg
    >
  </button>

  {#if isOpen}
    <div
      class="absolute right-0 mt-1 w-40 p-1 bg-surface border border-line-strong rounded-md overflow-hidden z-50"
    >
      <button
        onclick={() => selectTheme("light")}
        class="w-full h-[30px] flex items-center justify-between px-2 rounded-md text-sm text-primary hover:bg-active"
      >
        <div class="flex items-center gap-2">
          <span aria-hidden="true">{@html SunIcon}</span>
          <span>Light</span>
        </div>
        {#if $theme === "light"}
          <span aria-hidden="true">{@html CheckIcon}</span>
        {/if}
      </button>

      <button
        onclick={() => selectTheme("dark")}
        class="w-full h-[30px] flex items-center justify-between px-2 rounded-md text-sm text-primary hover:bg-active"
      >
        <div class="flex items-center gap-2">
          <span aria-hidden="true">{@html MoonIcon}</span>
          <span>Dark</span>
        </div>
        {#if $theme === "dark"}
          <span aria-hidden="true">{@html CheckIcon}</span>
        {/if}
      </button>


      <button
        onclick={() => selectTheme("system")}
        class="w-full h-[30px] flex items-center justify-between px-2 rounded-md text-sm text-primary hover:bg-active"
      >
        <div class="flex items-center gap-2">
          <span aria-hidden="true">{@html MonitorIcon}</span>
          <span>System</span>
        </div>
        {#if $theme === "system"}
          <span aria-hidden="true">{@html CheckIcon}</span>
        {/if}
      </button>
    </div>
  {/if}
</div>

<style>
  .theme-toggle-container {
    position: relative;
    z-index: 10;
  }
</style>

