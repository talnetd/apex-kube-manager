<script lang="ts">
  // Vercel-style tabs: muted labels, the active one primary with a 1px
  // underline. Scrolls sideways in a narrow side panel. Keyboard: one tab
  // stop (roving tabindex); Left/Right/Home/End move and activate.
  import { tick } from 'svelte';
  import { nextTabIndex } from '../../../utils/tabs';

  let {
    tabs,
    active,
    onchange,
    label = 'Sections',
  }: {
    tabs: { id: string; label: string; count?: number }[];
    active: string;
    onchange: (id: string) => void;
    /** Accessible name of the tablist. */
    label?: string;
  } = $props();

  let buttons: HTMLButtonElement[] = $state([]);

  // The one Tab stop: the active tab, or the first one when `active` names
  // no tab (a bad URL param, a tab this kind doesn't have), so the bar is
  // always reachable.
  const stop = $derived(tabs.some((t) => t.id === active) ? active : tabs[0]?.id);

  async function onKeydown(e: KeyboardEvent, i: number) {
    // Leave ⌥/⌘/Ctrl+arrows and Home/End to the OS and the app.
    if (e.altKey || e.metaKey || e.ctrlKey) return;
    const next = nextTabIndex(i, e.key, tabs.length);
    if (next === null) return;
    e.preventDefault();
    onchange(tabs[next].id);
    await tick();
    buttons[next]?.focus();
    buttons[next]?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  }
</script>

<div class="tabs" role="tablist" aria-label={label}>
  {#each tabs as t, i (t.id)}
    <button
      bind:this={buttons[i]}
      type="button"
      role="tab"
      id={`tab-${t.id}`}
      aria-selected={t.id === active}
      aria-controls={`panel-${t.id}`}
      tabindex={t.id === stop ? 0 : -1}
      class:on={t.id === active}
      onclick={() => onchange(t.id)}
      onkeydown={(e) => onKeydown(e, i)}
    >
      {t.label}{#if t.count !== undefined}<span class="c">{t.count}</span>{/if}
    </button>
  {/each}
</div>
<style>
  .tabs {
    display: flex;
    gap: 2px;
    padding: 0 12px;
    border-bottom: 1px solid var(--ui-line);
    overflow-x: auto;
    flex: none;
    scrollbar-width: none;
  }
  button {
    position: relative;
    height: 40px;
    padding: 0 10px;
    font-size: 13px;
    color: var(--ui-muted);
    white-space: nowrap;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  button:hover {
    color: var(--ui-primary);
  }
  button.on {
    color: var(--ui-primary);
    font-weight: 500;
  }
  button.on::after {
    content: '';
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: -1px;
    height: 1px;
    background: var(--ui-primary);
  }
  .c {
    font-size: 11px;
    color: var(--ui-muted);
    font-variant-numeric: tabular-nums;
  }
</style>
