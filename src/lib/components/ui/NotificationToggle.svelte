<script lang="ts">
  import { settings, toggleNotifications } from "../../stores/settings";
  import Icon from "../shell/Icon.svelte";
</script>

<button
  type="button"
  onclick={toggleNotifications}
  title={$settings.notificationsEnabled ? "Notifications on (click to mute)" : "Notifications muted (click to enable)"}
  aria-label={$settings.notificationsEnabled ? "Mute notifications" : "Enable notifications"}
  aria-pressed={!$settings.notificationsEnabled}
  class="w-7 h-7 inline-grid place-items-center rounded-md hover:bg-hover hover:text-primary {$settings.notificationsEnabled ? 'text-secondary' : 'text-faint'}"
>
  <Icon name={$settings.notificationsEnabled ? "bell" : "bellOff"} />
</button>
