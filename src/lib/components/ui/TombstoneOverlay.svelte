<script lang="ts">
  interface Props {
    resourceType: string;
    name: string;
    lastSeen?: string | null;
  }

  let { resourceType, name, lastSeen }: Props = $props();
</script>

<div class="absolute inset-0 bg-bg-primary/90 backdrop-blur-sm flex items-center justify-center z-50 rounded-lg">
  <div class="text-center p-8">
    <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-accent-error/20 flex items-center justify-center">
      <svg class="w-8 h-8 text-accent-error" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
    </div>
    <h3 class="text-xl font-semibold text-text-primary mb-2">{resourceType} Deleted</h3>
    <p class="text-text-secondary mb-2">
      <span class="font-mono text-accent-primary">{name}</span> no longer exists
    </p>
    {#if lastSeen}
      <p class="text-text-muted text-sm">Last seen: {lastSeen}</p>
    {/if}
    <p class="text-text-muted text-sm mt-4">This window shows the last known state</p>
  </div>
</div>
