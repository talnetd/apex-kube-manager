<script lang="ts">
  // Vercel-style tabs: muted labels, the active one primary with a 1px
  // underline. Scrolls sideways in a narrow side panel.
  let {
    tabs,
    active,
    onchange,
  }: {
    tabs: { id: string; label: string; count?: number }[];
    active: string;
    onchange: (id: string) => void;
  } = $props();
</script>

<!-- Plain toggle buttons (aria-pressed): a full tablist would need roving
     tabindex, arrow keys and tabpanels. -->
<div class="tabs">
  {#each tabs as t (t.id)}
    <button
      type="button"
      aria-pressed={t.id === active}
      class:on={t.id === active}
      onclick={() => onchange(t.id)}
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
