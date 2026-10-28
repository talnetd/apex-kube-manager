<script lang="ts">
  // Status bar (spec 4.4, 13.2; approved Geist preview): connection and
  // freshness always; context, version and node readiness from md up.
  import { onMount } from 'svelte';
  import { connectionStatus, currentContext, pulseMetrics, nodes, lastSyncedAt } from '../stores/kubernetes';
  import { currentContextIsProd } from '../stores/production';
  import { density, toggleDensity, densityLabel } from '../stores/density';
  import { activePortForwardCount, portForwardPanelOpen } from '../stores/portforward';
  import Icon from './shell/Icon.svelte';

  let now = $state(Date.now());
  onMount(() => {
    const id = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(id);
  });

  const ago = $derived($lastSyncedAt ? Math.max(0, Math.round((now - $lastSyncedAt) / 1000)) : null);
  const agoText = $derived(ago === null ? 'not synced yet' : ago < 60 ? `synced ${ago}s ago` : `synced ${Math.round(ago / 60)}m ago`);

  // Node readiness from the node watch when it is running, else just the count.
  const nodeText = $derived.by(() => {
    if ($nodes.length) {
      const ready = $nodes.filter((n) => n.verdict.status === 'ok').length;
      return { text: `${ready}/${$nodes.length} nodes ready`, warn: ready < $nodes.length };
    }
    if ($pulseMetrics) return { text: `${$pulseMetrics.nodes} nodes`, warn: false };
    return null;
  });

  const connText = $derived(
    $connectionStatus === 'connected' ? 'Connected' : $connectionStatus === 'connecting' ? 'Connecting' : 'Disconnected'
  );
</script>

<footer class="status">
  <span class="flex items-center gap-1.5">
    <span class="sdot" class:bad={$connectionStatus === 'disconnected'} aria-hidden="true"></span>
    <span class="text-secondary">{connText}</span>
  </span>
  {#if $currentContextIsProd}
    <span class="flex items-center gap-1 text-warning hide-s"><Icon name="shield" size={14} />{$currentContext} · Production</span>
  {:else if $currentContext}
    <span class="hide-s">{$currentContext}</span>
  {/if}
  {#if $pulseMetrics?.k8s_version}<span class="hide-s font-mono">{$pulseMetrics.k8s_version}</span>{/if}
  {#if nodeText}<span class="hide-s" class:text-warning={nodeText.warn}>{nodeText.text}</span>{/if}
  <span class="tabular-nums">live · {agoText}</span>
  <span class="flex-1"></span>
  <button type="button" class="hide-s hover:text-primary" title="Density · ⌘⇧D" onclick={toggleDensity}>{densityLabel($density)}</button>
  {#if $activePortForwardCount > 0}
    <button type="button" class="hide-s hover:text-primary" onclick={() => portForwardPanelOpen.set(true)}
      >⇄ {$activePortForwardCount} {$activePortForwardCount === 1 ? 'forward' : 'forwards'}</button
    >
  {/if}
</footer>

<style>
  .status {
    grid-column: 1 / 3;
    grid-row: 3;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 12px;
    background: var(--ui-app);
    border-top: 1px solid var(--ui-line);
    font-size: 12px;
    line-height: 16px;
    color: var(--ui-muted);
    white-space: nowrap;
    overflow: hidden;
  }
  .sdot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ui-faint);
    flex: none;
  }
  .sdot.bad {
    background: var(--ui-danger-solid);
  }
  @media (width < 640px) {
    .status {
      grid-column: 1;
    }
    .hide-s {
      display: none;
    }
  }
</style>
