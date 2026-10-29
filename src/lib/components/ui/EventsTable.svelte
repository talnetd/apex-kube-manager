<script lang="ts">
  // Events as hairline rows like DataTable: type as an icon + word (only
  // Warning is coloured), reason in mono, message under it, time right.
  import StatusPill from './StatusPill.svelte';
  import Button from './Button.svelte';
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


</script>

<div class="h-full flex flex-col">
  <div class="ebar">
    <span class="text-sm text-secondary">{events.length} {events.length === 1 ? 'event' : 'events'}{#if isDeleted}<span class="text-muted"> · cached</span>{/if}</span>
    {#if onRefresh}<Button size="sm" disabled={isDeleted} onclick={onRefresh}>Refresh</Button>{/if}
  </div>

  <div class="flex-1 overflow-auto">
    {#if isLoading}
      <p class="text-muted text-sm px-5 py-8">Loading events…</p>
    {:else if events.length > 0}
      <ul class="elist">
        {#each events as event}
          {@const warn = getEventType(event) === 'Warning'}
          <li>
            <div class="l1">
              <StatusPill health={warn ? 'warning' : 'neutral'} label={getEventType(event)} variant="dot" />
              <span class="reason">{event.reason}</span>
              {#if event.count > 1}<span class="count">×{event.count}</span>{/if}
              <span class="time">{formatTime(event.last_timestamp) || formatTime(event.first_timestamp)}</span>
            </div>
            <p class="msg">{event.message}</p>
            {#if event.source}<p class="src">{event.source}</p>{/if}
          </li>
        {/each}
      </ul>
    {:else}
      <p class="text-muted text-sm px-5 py-8">No events for this {resourceType.toLowerCase()} in the last hour or so (the API server keeps them briefly).</p>
    {/if}
  </div>
</div>

<style>
  .ebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 20px;
    border-bottom: 1px solid var(--ui-line);
  }
  .elist {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .elist li {
    padding: 10px 20px;
    border-bottom: 1px solid var(--ui-line);
  }
  .l1 {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
    flex-wrap: wrap;
  }
  .reason {
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: var(--ui-primary);
  }
  .count {
    font-size: 12px;
    color: var(--ui-muted);
    font-variant-numeric: tabular-nums;
  }
  .time {
    margin-left: auto;
    font-size: 12px;
    color: var(--ui-muted);
    font-variant-numeric: tabular-nums;
  }
  .msg {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--ui-secondary);
    overflow-wrap: anywhere;
  }
  .src {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--ui-muted);
  }
  @media (max-width: 639.98px) {
    .ebar,
    .elist li {
      padding-left: 12px;
      padding-right: 12px;
    }
  }
</style>
