<script lang="ts">
  // Menu bar popover (spec 13.5): clusters with a status dot, switch the
  // shared context (prod guarded), open Apex. No restart/scale/delete here.
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import Toaster from '../ui/Toaster.svelte';
  import { settings } from '../../stores/settings';
  import { resolveProd } from '../../stores/production';
  import { toast } from '../../stores/toast';
  import { DOT_LABEL, clusterNote, sinceLabel, traySummary, type ClusterStatus, type TraySnapshot } from '../../utils/tray';

  let snap = $state<TraySnapshot>({ enabled: true, clusters: [], checked_at: null });
  let now = $state(Date.now());
  let refreshing = $state(false);
  let confirmCtx = $state<string | null>(null);
  let busy = $state<string | null>(null);

  const isProd = (ctx: string) => resolveProd(ctx, $settings.prodOverrides);

  onMount(() => {
    invoke<TraySnapshot>('get_tray_status').then((s) => (snap = s)).catch(() => {});
    const offs = [
      listen<TraySnapshot>('tray-status', (e) => (snap = e.payload)),
      // A switch from a main window: Rust re-polls and sends tray-status; this
      // just keeps the "current" check honest until then.
      listen<{ context: string }>('context-changed', (e) => {
        snap = { ...snap, clusters: snap.clusters.map((c) => ({ ...c, current: c.context === e.payload.context })) };
      }),
    ];
    const clock = setInterval(() => (now = Date.now()), 1000);
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !confirmCtx) invoke('hide_tray_popover');
    };
    window.addEventListener('keydown', onKey);
    return () => {
      offs.forEach((p) => p.then((off) => off()));
      clearInterval(clock);
      window.removeEventListener('keydown', onKey);
    };
  });

  async function refresh() {
    refreshing = true;
    try {
      await invoke('refresh_tray_status');
    } finally {
      refreshing = false;
    }
  }

  async function doSwitch(ctx: string) {
    busy = ctx;
    try {
      await invoke('switch_context', { contextName: ctx });
      await invoke('focus_main_window');
      await invoke('hide_tray_popover');
    } catch (e) {
      toast(`Couldn't switch to ${ctx}: ${e}`, { health: 'failed' });
    } finally {
      busy = null;
    }
  }

  // Production contexts go through the shared prod-guarded confirm (spec 4.6).
  function requestSwitch(c: ClusterStatus) {
    if (isProd(c.context)) confirmCtx = c.context;
    else doSwitch(c.context);
  }

  async function openApex() {
    await invoke('focus_main_window');
    await invoke('hide_tray_popover');
  }

  const summary = $derived(traySummary(snap.clusters));
  const since = $derived(sinceLabel(snap.checked_at, now));
  const anyBad = $derived(snap.clusters.some((c) => c.dot === 'red' || c.dot === 'amber'));
</script>

<div class="pop">
  <header class="hd">
    <div class="t1">
      <span class="logo" aria-hidden="true"></span>
      <span class="font-semibold">Apex</span>
      <span class="ml-auto text-xs text-muted tabular-nums">{since ? `synced ${since} ago` : ''}</span>
      <button type="button" class="ib" aria-label="Check now" title="Check now" disabled={refreshing} onclick={refresh}>
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="M13 8a5 5 0 1 1-1.5-3.6M13 2.5v3h-3" /></svg>
      </button>
    </div>
    <div class="t2" class:text-primary={anyBad}>{summary}</div>
  </header>

  <div class="body">
    <h4>Clusters</h4>
    {#each snap.clusters as c (c.context)}
      <div class="row" title={c.error ?? undefined}>
        <span class="dot d-{c.dot}" role="img" aria-label={DOT_LABEL[c.dot]}></span>
        <span class="min-w-0 flex-1">
          <span class="flex items-center gap-1.5 min-w-0">
            <span class="nm">{c.context}</span>
            {#if isProd(c.context)}<span class="prodtag">PROD</span>{/if}
          </span>
          <span class="sub">{clusterNote(c)}</span>
        </span>
        {#if c.current}
          <span class="cur">current</span>
        {:else}
          <button
            type="button"
            class="sw"
            title={`Switch all windows to ${c.context}`}
            disabled={busy !== null}
            onclick={() => requestSwitch(c)}>{busy === c.context ? 'Switching…' : 'Switch to'}</button
          >
        {/if}
      </div>
    {:else}
      <div class="row text-muted">No contexts in your kubeconfig.</div>
    {/each}
  </div>

  <footer class="ft">
    <button type="button" class="primary" onclick={openApex}>Open Apex</button>
    <button type="button" class="ghost" onclick={() => invoke('set_tray_enabled', { enabled: false })}>Hide from menu bar</button>
  </footer>
</div>

{#if confirmCtx}
  {@const ctx = confirmCtx}
  <ConfirmDialog
    open={true}
    title={`Switch to ${ctx}?`}
    resource={{ kind: 'Context', name: ctx, context: ctx }}
    confirmLabel="Switch"
    body="Every Apex window will switch to this production context."
    onConfirm={() => doSwitch(ctx)}
    onClose={() => (confirmCtx = null)}
  />
{/if}

<Toaster />

<style>
  .pop {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--ui-surface);
    border: 1px solid var(--ui-line-strong);
    border-radius: 10px;
    overflow: hidden;
    font-size: 13px;
    color: var(--ui-primary);
  }
  .hd {
    padding: 12px 14px 10px;
    border-bottom: 1px solid var(--ui-divider);
  }
  .t1 {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .t2 {
    margin-top: 2px;
    color: var(--ui-secondary);
  }
  .logo {
    width: 0;
    height: 0;
    border-left: 6px solid transparent;
    border-right: 6px solid transparent;
    border-bottom: 11px solid var(--ui-primary);
  }
  .ib {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: 6px;
    color: var(--ui-secondary);
  }
  .ib:hover {
    background: var(--ui-hover);
    color: var(--ui-primary);
  }
  .body {
    flex: 1;
    overflow: auto;
    padding: 8px 0 4px;
  }
  h4 {
    margin: 0;
    padding: 0 14px 4px;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--ui-muted);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    min-width: 0;
  }
  .row:hover {
    background: var(--ui-hover);
  }
  .nm {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .sub {
    display: block;
    font-size: 11.5px;
    color: var(--ui-muted);
  }
  /* The one place green is allowed (spec 13.1): green-vs-red is the point. */
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex: none;
  }
  .d-green {
    background: var(--ui-success-solid);
  }
  .d-amber {
    background: var(--ui-warning-solid);
  }
  .d-red {
    background: var(--ui-danger-solid);
  }
  .d-grey {
    box-shadow: inset 0 0 0 1.5px var(--ui-faint);
  }
  .cur {
    font-size: 11px;
    color: var(--ui-muted);
  }
  .sw {
    height: 22px;
    padding: 0 8px;
    border-radius: 6px;
    border: 1px solid var(--ui-line-strong);
    font-size: 12px;
    white-space: nowrap;
  }
  .sw:hover:not(:disabled) {
    background: var(--ui-hover);
  }
  .sw:disabled {
    opacity: 0.4;
  }
  .ft {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-top: 1px solid var(--ui-divider);
  }
  .primary {
    height: 26px;
    padding: 0 10px;
    border-radius: 6px;
    background: var(--ui-btn);
    color: var(--ui-on-btn);
    font-weight: 500;
  }
  .primary:hover {
    opacity: 0.88;
  }
  .ghost {
    margin-left: auto;
    font-size: 12px;
    color: var(--ui-muted);
  }
  .ghost:hover {
    color: var(--ui-primary);
  }
</style>
