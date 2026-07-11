<script lang="ts">
  import { settings, toggleNotifications } from "../../stores/settings";
</script>

<button
  onclick={toggleNotifications}
  title={$settings.notificationsEnabled ? "Notifications enabled (click to mute)" : "Notifications muted (click to enable)"}
  class="h-9 w-9 inline-flex items-center justify-center rounded-lg
      bg-transparent hover:bg-bg-tertiary transition-colors
      {$settings.notificationsEnabled ? 'text-text-secondary hover:text-text-primary' : 'text-text-muted hover:text-text-secondary'}
      focus:outline-none"
>
  {#if $settings.notificationsEnabled}
    <!-- Bell icon -->
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
        d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
    </svg>
  {:else}
    <!-- Bell slash icon -->
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
        d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
      <line x1="3" y1="3" x2="21" y2="21" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    </svg>
  {/if}
</button>
