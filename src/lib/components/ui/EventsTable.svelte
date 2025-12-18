<script lang="ts">
  interface ResourceEvent {
    event_type?: string;
    type?: string;
    reason: string;
    message: string;
    count: number;
    first_timestamp?: string | null;
    last_timestamp?: string | null;
    source?: string;
  }

  interface Props {
    events: ResourceEvent[];
    resourceType?: string;
    isLoading?: boolean;
    isDeleted?: boolean;
    onRefresh?: () => void;
  }

  let { events, resourceType = 'Resource', isLoading = false, isDeleted = false, onRefresh }: Props = $props();

  function formatTime(time: string | null | undefined): string {
    if (!time) return '';
    const date = new Date(time);
    return date.toLocaleString();
  }

  function getEventType(e: ResourceEvent): string {
    return e.event_type || e.type || 'Normal';
  }

  function getEventTypeColor(type: string): string {
    return type === 'Warning' ? 'bg-accent-warning/20 text-accent-warning' : 'bg-accent-success/20 text-accent-success';
  }
</script>

<div class="h-full flex flex-col">
  <!-- Header Bar -->
  <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
    <div class="flex items-center gap-2">
      <span class="text-sm text-text-muted">{resourceType} Events</span>
      {#if isDeleted}
        <span class="text-xs text-accent-warning">(cached)</span>
      {/if}
    </div>
    {#if onRefresh}
      <button
        onclick={onRefresh}
        disabled={isDeleted}
        class="text-xs px-3 py-1 bg-bg-tertiary rounded hover:bg-border-subtle transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        Refresh
      </button>
    {/if}
  </div>

  <!-- Events Content -->
  <div class="flex-1 overflow-auto p-4">
    {#if isLoading}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <svg class="w-8 h-8 text-accent-primary mx-auto mb-2 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <p class="text-text-muted">Loading events...</p>
        </div>
      </div>
    {:else if events.length > 0}
      <div class="space-y-3">
        {#each events as event}
          <div class="bg-bg-secondary rounded-lg p-4">
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2">
                <span class="text-xs px-2 py-0.5 rounded {getEventTypeColor(getEventType(event))}">{getEventType(event)}</span>
                <span class="font-medium text-text-primary">{event.reason}</span>
                {#if event.count > 1}
                  <span class="text-xs text-text-muted">x{event.count}</span>
                {/if}
              </div>
              <span class="text-xs text-text-muted">{formatTime(event.last_timestamp) || formatTime(event.first_timestamp)}</span>
            </div>
            <p class="text-sm text-text-secondary">{event.message}</p>
            {#if event.source}
              <p class="text-xs text-text-muted mt-1">Source: {event.source}</p>
            {/if}
          </div>
        {/each}
      </div>
    {:else}
      <div class="flex items-center justify-center h-full">
        <p class="text-text-muted">No events found</p>
      </div>
    {/if}
  </div>
</div>
