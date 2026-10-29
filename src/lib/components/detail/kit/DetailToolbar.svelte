<script lang="ts">
  // The status/actions row under a detail window's title bar: the triage
  // verdict as a StatusPill (icon + reason) on the left, actions on the
  // right (primary neutral, destructive outlined red). Wraps at 400px.
  import type { Snippet } from 'svelte';
  import StatusPill from '../../ui/StatusPill.svelte';
  import { toDisplay, type Verdict } from '../../../utils/health';

  let {
    verdict,
    summary,
    actions,
  }: {
    verdict?: Verdict | null;
    /** Short facts next to the status, e.g. "3/3 ready · rev 12". */
    summary?: string;
    actions?: Snippet;
  } = $props();

  const h = $derived(verdict ? toDisplay(verdict) : null);
</script>

{#if h || summary || actions}
<div class="bar">
  <div class="left">
    {#if h}<StatusPill health={h.health} label={h.label || verdict?.reason || ''} variant="dot" title={verdict?.ruleId ? `${verdict.reason} (${verdict.ruleId})` : undefined} />{/if}
    {#if summary}<span class="sum">{summary}</span>{/if}
  </div>
  {#if actions}<div class="acts">{@render actions()}</div>{/if}
</div>
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px 16px;
    min-height: 44px;
    padding: 6px 20px;
    border-bottom: 1px solid var(--ui-line);
    flex: none;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  .sum {
    font-size: 12px;
    color: var(--ui-muted);
    font-variant-numeric: tabular-nums;
  }
  .acts {
    margin-left: auto;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  @media (max-width: 639.98px) {
    .bar {
      padding: 6px 12px;
    }
  }
</style>
