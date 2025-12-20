<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';

  const appWindow = getCurrentWindow();

  async function minimizeWindow() {
    await appWindow.minimize();
  }

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
  }

  async function closeWindow() {
    await appWindow.close();
  }
</script>

<div class="flex items-center gap-1">
  <!-- Minimize -->
  <button
    onclick={minimizeWindow}
    class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-primary transition-colors"
    title="Minimize"
  >
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
    </svg>
  </button>
  <!-- Maximize -->
  <button
    onclick={toggleMaximize}
    class="p-1.5 rounded hover:bg-bg-tertiary text-text-muted hover:text-accent-primary transition-colors"
    title="Maximize"
  >
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <rect x="4" y="4" width="16" height="16" rx="1" stroke-width="2" />
    </svg>
  </button>
  <!-- Close -->
  <button
    onclick={closeWindow}
    class="p-1.5 rounded hover:bg-accent-error/20 text-text-muted hover:text-accent-error transition-colors"
    title="Close"
  >
    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
    </svg>
  </button>
</div>
