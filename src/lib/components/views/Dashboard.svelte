<script lang="ts">
  // Pulse (spec 6, approved Geist preview v5): KPI strip, health grid,
  // CPU/Memory/Disk with 1h history, activity feed and the issues table with
  // a side sheet. Health is the Rust triage verdict on each object.
  import './pulse/pulse.css';
  import { onMount, onDestroy, untrack } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    currentContext,
    selectedNamespace,
    refreshTrigger,
    pulseMetrics,
    pods,
    clusterEvents,
    lastSyncedAt,
    apiProbes,
    loadPulseMetrics,
    loadStatefulSets,
    loadDaemonSets,
    loadReplicaSets,
    loadNodeMetrics,
    startDeploymentWatch,
    stopDeploymentWatch,
    startJobWatch,
    stopJobWatch,
    startNodeWatch,
    stopNodeWatch,
    startPodWatch,
    stopPodWatch,
    nodeMetrics,
    type PodInfo,
  } from '../../stores/kubernetes';
  import { selectedRowIndex, totalRows, resetNavigation, keyboardNavActive } from '../../stores/keyboard';
  import { currentContextIsProd } from '../../stores/production';
  import { toast } from '../../stores/toast';
  import { isTypingTarget } from '../../utils/keys';
  import { toDisplay, type Health } from '../../utils/health';
  import StatusPill from '../ui/StatusPill.svelte';
  import StatusIcon from '../ui/StatusIcon.svelte';
  import ConfirmDialog from '../ui/ConfirmDialog.svelte';
  import Icon from '../shell/Icon.svelte';
  import type { IconName } from '../shell/icons';
  import LineSpark from './pulse/LineSpark.svelte';
  import IssueSheet from './pulse/IssueSheet.svelte';
  import {
    initPulse,
    workloads,
    nodeObjects,
    kpis,
    activity,
    historyTick,
    lastHour,
    firstSeen,
    recordAction,
    worstDisk,
  } from '../../pulse/store';
  import { delta } from '../../pulse/history';
  import {
    buildIssues,
    podsFor,
    eventsFor,
    findTrigger,
    reachabilityPct,
    shortDuration,
    objectKey,
    KIND_SHORT,
    SEV_LABEL,
    type Issue,
    type PulseObject,
    type ActivityItem,
  } from '../../pulse/model';

  initPulse();

  // ---------- Data: watches (push) plus the existing 5s metrics poll ----------
  const POLL_MS = 5000;
  /** Lists without a watch ride on every 6th poll tick (30s). */
  const SLOW_EVERY = 6;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let clockTimer: ReturnType<typeof setInterval> | undefined;
  let tickN = 0;
  let now = $state(Date.now());

  function loadSlow(ns: string | null) {
    loadStatefulSets(ns);
    loadDaemonSets(ns);
    loadReplicaSets(ns);
    loadNodeMetrics();
  }

  // Runs on mount too, so it is the only place watches start (starting twice
  // concurrently would leak the first stream).
  $effect(() => {
    const ctx = $currentContext;
    const ns = $selectedNamespace;
    void $refreshTrigger;
    if (!ctx) return;
    untrack(() => {
      startDeploymentWatch(ns);
      startJobWatch(ns);
      startPodWatch(ns);
      startNodeWatch().then(() => loadNodeMetrics());
      loadPulseMetrics(ns);
      loadSlow(ns);
    });
  });

  onMount(() => {
    resetNavigation();
    // 1s clock for "synced Ns ago" and ages; no requests.
    clockTimer = setInterval(() => (now = Date.now()), 1000);
    pollTimer = setInterval(() => {
      const ns = $selectedNamespace;
      loadPulseMetrics(ns);
      if (++tickN % SLOW_EVERY === 0) loadSlow(ns);
    }, POLL_MS);
    window.addEventListener('keydown', onKeydown);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
    if (clockTimer) clearInterval(clockTimer);
    window.removeEventListener('keydown', onKeydown);
    stopDeploymentWatch();
    stopJobWatch();
    stopPodWatch();
    stopNodeWatch();
    resetNavigation();
  });

  // ---------- Derived state ----------
  const objects = $derived<PulseObject[]>([...$workloads, ...$nodeObjects]);
  const issues = $derived(buildIssues(objects, firstSeen));
  const issueById = $derived(new Map(issues.map((i) => [i.id, i])));

  const podsOf = (i: Issue) => podsFor(i, $pods).filter((p) => i.kind === 'node' || p.verdict.status !== 'ok');
  const nodesOf = (i: Issue) => [...new Set(podsFor(i, $pods).map((p) => p.node).filter((n): n is string => !!n))];
  const triggerOf = (i: Issue) => findTrigger(i, $activity, nodesOf(i));

  function impactOf(i: Issue): string {
    const ps = podsOf(i);
    if (i.kind === 'node') return ps.length ? `${ps.length} pods scheduled on this node` : 'No pods on this node';
    const parts: string[] = [];
    if (i.ready) parts.push(`${i.ready} ready`);
    if (ps.length) parts.push(`${ps.length} ${ps.length === 1 ? 'pod' : 'pods'} unhealthy`);
    return parts.join(' · ') || i.reason;
  }

  function triggerShort(i: Issue): string | null {
    const t = triggerOf(i);
    if (!t) return null;
    const what = t.item.kind === 'rollout' ? 'rollout' : t.item.kind === 'scale' ? 'scale change' : t.item.text.charAt(0).toLowerCase() + t.item.text.slice(1);
    return `started ${shortDuration(t.afterMs)} after ${what}`;
  }

  // History (re-read on every sample)
  const series = $derived.by(() => {
    void $historyTick;
    const at = Date.now();
    return {
      cpu: lastHour('cpu', at),
      mem: lastHour('mem', at),
      disk: lastHour('disk', at),
      health: lastHour('health', at),
      workloads: lastHour('workloads', at),
    };
  });
  const vals = (s: { v: number | null }[]) => s.map((x) => x.v);

  const reach = $derived(reachabilityPct($apiProbes.ok, $apiProbes.total));
  const healthDelta = $derived(delta(series.health));
  const workloadDelta = $derived(delta(series.workloads));

  // Capacity numbers for the metric tiles
  const pct = (used: number | null | undefined, of: number | undefined) =>
    used != null && of ? Math.min(100, (used / of) * 100) : null;
  const cpuPct = $derived($pulseMetrics?.metrics_available ? pct($pulseMetrics.cpu_usage, $pulseMetrics.cpu_allocatable) : null);
  const memPct = $derived($pulseMetrics?.metrics_available ? pct($pulseMetrics.memory_usage, $pulseMetrics.memory_allocatable) : null);
  const diskPct = $derived(worstDisk($nodeMetrics));
  const worstDiskNode = $derived.by(() => {
    let best: { name: string; p: number } | null = null;
    for (const m of Object.values($nodeMetrics)) {
      if (m.disk_usage == null || !m.disk_capacity) continue;
      const p = (m.disk_usage / m.disk_capacity) * 100;
      if (!best || p > best.p) best = { name: m.name, p };
    }
    return best?.name ?? null;
  });
  const cores = (m: number) => (m / 1000).toFixed(1);
  const gib = (b: number) => (b / 1024 ** 3).toFixed(0);

  const syncedAgo = $derived($lastSyncedAt ? Math.max(0, Math.round((now - $lastSyncedAt) / 1000)) : null);

  // ---------- Health grid ----------
  let gridMode = $state<'workloads' | 'nodes'>('workloads');
  const SQUARE: Record<string, 'healthy' | 'warning' | 'failed' | 'unknown'> = { ok: 'healthy', warn: 'warning', fail: 'failed', unknown: 'unknown' };
  const gridItems = $derived(
    (gridMode === 'nodes' ? $nodeObjects : $workloads)
      .slice()
      .sort((a, b) => a.name.localeCompare(b.name) || a.kind.localeCompare(b.kind))
  );
  const gridCount = (s: string) => gridItems.filter((o) => SQUARE[o.verdict.status] === s).length;

  let tip = $state<{ x: number; y: number; obj: PulseObject; below: boolean } | null>(null);
  function showTip(e: Event, obj: PulseObject) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    tip = { x: r.left + r.width / 2, y: r.top < 80 ? r.bottom + 8 : r.top - 8, obj, below: r.top < 80 };
  }

  // ---------- Selection, sheet, keyboard ----------
  let sheetOpen = $state(false);
  const selected = $derived($selectedRowIndex >= 0 && $selectedRowIndex < issues.length ? issues[$selectedRowIndex] : null);

  $effect(() => {
    totalRows.set(issues.length);
  });
  // Sheet closes when its issue resolves.
  $effect(() => {
    if (sheetOpen && !selected) sheetOpen = false;
  });

  let rowEls: HTMLTableRowElement[] = $state([]);
  $effect(() => {
    const i = $selectedRowIndex;
    if ($keyboardNavActive && i >= 0) rowEls[i]?.scrollIntoView({ block: 'nearest' });
  });

  function openIssue(i: Issue) {
    const idx = issues.findIndex((x) => x.id === i.id);
    if (idx >= 0) selectedRowIndex.set(idx);
    sheetOpen = true;
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.defaultPrevented || e.metaKey || e.ctrlKey || e.altKey) return;
    if (isTypingTarget(document.activeElement) || confirm) return;
    if (e.key === 'Escape' && sheetOpen) {
      e.preventDefault();
      sheetOpen = false;
      return;
    }
    if (e.key === 'Enter' && selected && !sheetOpen) {
      if ((document.activeElement as HTMLElement | null)?.tagName === 'BUTTON') return;
      e.preventDefault();
      sheetOpen = true;
    }
  }

  // ---------- Actions ----------
  const RESOURCE: Record<string, string> = { deployment: 'deployment', statefulset: 'statefulset', daemonset: 'daemonset', job: 'job', node: 'node' };

  async function openDetail(kind: string, name: string, namespace: string | null, tab?: string) {
    try {
      await invoke('open_resource_detail', {
        resourceType: kind,
        name,
        namespace: kind === 'node' ? '' : (namespace ?? ''),
        context: $currentContext,
        tab,
      });
    } catch (e) {
      toast(`Couldn't open ${name}`, { health: 'failed' });
      console.error(e);
    }
  }
  const openObject = (o: { kind: string; name: string; namespace: string | null }) => openDetail(RESOURCE[o.kind], o.name, o.namespace);
  const openPod = (p: PodInfo, tab?: string) => openDetail('pod', p.name, p.namespace, tab);
  const logsTarget = (i: Issue) => (i.kind === 'node' ? null : (podsOf(i)[0] ?? null));
  const canRestart = (i: Issue) => i.kind === 'deployment' || i.kind === 'statefulset';

  function copyRef(i: Issue) {
    const ref = `${KIND_SHORT[i.kind]}/${i.name}`;
    const cmd = `kubectl ${i.namespace ? `-n ${i.namespace} ` : ''}describe ${ref} --context ${$currentContext}`;
    navigator.clipboard?.writeText(cmd).then(
      () => toast('Copied kubectl describe'),
      () => toast("Couldn't copy to clipboard", { health: 'failed' })
    );
  }

  type ConfirmState = { mode: 'restart' | 'scale'; issue: Issue };
  let confirm = $state<ConfirmState | null>(null);
  let replicas = $state(1);

  function askRestart(i: Issue) {
    confirm = { mode: 'restart', issue: i };
  }
  function askScale(i: Issue) {
    const desired = Number((i.ready ?? '').split('/')[1]);
    replicas = Number.isFinite(desired) ? desired : 1;
    confirm = { mode: 'scale', issue: i };
  }

  async function runConfirm() {
    if (!confirm) return;
    const { mode, issue: i } = confirm;
    const cmd = `${mode}_${i.kind}`;
    if (mode === 'restart') {
      await invoke(cmd, { namespace: i.namespace, name: i.name });
    } else {
      await invoke(cmd, { namespace: i.namespace, name: i.name, replicas });
    }
    recordAction({
      kind: 'action',
      objectKind: i.kind,
      target: { kind: i.kind, name: i.name, namespace: i.namespace },
      text: mode === 'restart' ? `Restart ${KIND_SHORT[i.kind]}/${i.name}` : `Scale ${KIND_SHORT[i.kind]}/${i.name} to ${replicas}`,
      detail: i.namespace ?? '',
    });
  }

  // ---------- Activity ----------
  let actOpen = $state(false);
  function actIcon(a: ActivityItem): { cls: string; icon?: IconName; status?: Health } {
    if (a.tone === 'apex') return { cls: 'a-apex', icon: 'restart' };
    if (a.kind === 'warning') return { cls: 'a-warn', status: 'warning' };
    if (a.kind === 'node') {
      if (a.text.startsWith('Cordoned')) return { cls: 'a-warn', icon: 'cordon' };
      return a.tone === 'bad' ? { cls: 'a-bad', status: 'failed' } : { cls: '', icon: 'node' };
    }
    if (a.kind === 'restart') return { cls: '', icon: 'restart' };
    if (a.kind === 'scale') return { cls: '', icon: 'scale' };
    return { cls: '', icon: 'deploy' };
  }
  const ago = (t: number) => {
    const m = Math.max(0, Math.round((now - t) / 60000));
    return m < 1 ? 'just now' : `${m}m ago`;
  };
  const triggerIds = $derived(new Set(selected && sheetOpen ? [triggerOf(selected)?.item.id].filter(Boolean) : []));

  const fmtPct = (v: number | null, digits = 1) => (v === null ? '–' : `${v.toFixed(digits)}%`);
  const nsLabel = $derived($selectedNamespace ?? 'all namespaces');
</script>

<div class="pulse">
  <div class="page">
    <div class="p4h">
      <h1>Pulse</h1>
      <span class="sub">
        <span>{$currentContext}</span>
        {#if $currentContextIsProd}<span class="prodtag">PROD</span>{/if}
        <span>· {nsLabel}{syncedAgo !== null ? ` · synced ${syncedAgo}s ago` : ''}</span>
      </span>
      <div class="right">
        <button
          type="button"
          class="btn"
          onclick={() => {
            loadPulseMetrics($selectedNamespace);
            loadSlow($selectedNamespace);
          }}><Icon name="restart" size={14} />Refresh</button
        >
      </div>
    </div>

    <!-- KPI strip -->
    <div class="kpis">
      <section class="kpi" class:k-warn={$kpis.healthPct !== null && $kpis.healthPct < 95} aria-label="Health {fmtPct($kpis.healthPct)}">
        <div class="kl">Health</div>
        <div class="kv2">{fmtPct($kpis.healthPct)}</div>
        <div class="kd">
          {#if series.health.length > 1}<LineSpark values={vals(series.health)} w={64} h={18} />{/if}
          <span>{healthDelta === null ? 'healthy / total' : Math.abs(healthDelta) < 0.05 ? 'no change · 1h' : `${healthDelta > 0 ? '↑' : '↓'} ${Math.abs(healthDelta).toFixed(1)} pts · 1h`}</span>
        </div>
      </section>
      <section class="kpi" aria-label="Active workloads {$kpis.total}">
        <div class="kl">Active workloads</div>
        <div class="kv2">{$kpis.total}</div>
        <div class="kd">
          <span>{workloadDelta === null || workloadDelta === 0 ? 'no change' : `${workloadDelta > 0 ? '+' : ''}${workloadDelta} in the last hour`}</span>
        </div>
      </section>
      <section class="kpi" aria-label="Healthy {$kpis.healthy}">
        <div class="kl"><StatusIcon health="healthy" /> Healthy</div>
        <div class="kv2">{$kpis.healthy}</div>
        <div class="kd"><span>{$kpis.total ? Math.round(($kpis.healthy / $kpis.total) * 100) : 0}% of workloads</span></div>
      </section>
      <section class="kpi" class:k-warn={$kpis.warning > 0} aria-label="Warning {$kpis.warning}">
        <div class="kl"><StatusIcon health="warning" /> Warning</div>
        <div class="kv2">{$kpis.warning}</div>
        <div class="kd"><span>{$kpis.warning ? 'needs a look' : 'none'}</span></div>
      </section>
      <section class="kpi" class:k-crit={$kpis.critical > 0} aria-label="Critical {$kpis.critical}">
        <div class="kl"><StatusIcon health="failed" /> Critical</div>
        <div class="kv2">{$kpis.critical}</div>
        <div class="kd"><span>{$kpis.critical ? 'act now' : 'none'}</span></div>
      </section>
      <section class="kpi" aria-label="Uptime {fmtPct(reach, 2)}">
        <div class="kl">Uptime</div>
        <div class="kv2">{reach === null ? '–' : reach === 100 ? '100%' : fmtPct(reach, 2)}</div>
        <div class="kd"><span>API server · this session</span></div>
      </section>
    </div>

    <div class="mgrid">
      <div class="mleft">
        <!-- Health grid -->
        <section class="box" aria-label="Health grid">
          <div class="bh">
            <h2>Health grid</h2>
            <span class="cnt">{gridItems.length} {gridMode}</span>
            <div class="r">
              <div class="seg" role="group" aria-label="Show">
                <button type="button" aria-pressed={gridMode === 'workloads'} onclick={() => (gridMode = 'workloads')}>Workloads</button>
                <button type="button" aria-pressed={gridMode === 'nodes'} onclick={() => (gridMode = 'nodes')}>Nodes</button>
              </div>
            </div>
          </div>
          {#if gridItems.length}
            <div
              class="hgrid"
              class:nodes={gridMode === 'nodes'}
              role="group"
              aria-label="{gridCount('healthy')} healthy, {gridCount('warning')} warning, {gridCount('failed')} critical, {gridCount('unknown')} unknown"
            >
              {#each gridItems as o (objectKey(o))}
                {@const s = SQUARE[o.verdict.status]}
                {@const issue = issueById.get(objectKey(o))}
                <button
                  type="button"
                  class="hc s-{s}"
                  tabindex="-1"
                  aria-label="{KIND_SHORT[o.kind]}/{o.name}, {o.verdict.reason || s}"
                  onmouseenter={(e) => showTip(e, o)}
                  onmouseleave={() => (tip = null)}
                  onfocus={(e) => showTip(e, o)}
                  onblur={() => (tip = null)}
                  onclick={() => (issue ? openIssue(issue) : openObject(o))}
                >
                  {#if s === 'failed'}
                    <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><path d="M3 3l4 4M7 3L3 7" /></svg>
                  {:else if s === 'warning'}
                    <svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true"><path d="M5 2.4v3.2M5 7.6v.1" /></svg>
                  {/if}
                </button>
              {/each}
            </div>
            <div class="legend">
              <span><i class="s-healthy"></i>Healthy <b>{gridCount('healthy')}</b></span>
              <span><i class="s-warning"></i>Warning <b>{gridCount('warning')}</b></span>
              <span><i class="s-failed"></i>Critical <b>{gridCount('failed')}</b></span>
              <span><i class="s-unknown"></i>Unknown <b>{gridCount('unknown')}</b></span>
              <span class="muted hide-s" style="margin-left:auto">hover for details · click to open</span>
            </div>
          {:else}
            <div class="empty-grid">Loading {gridMode}…</div>
          {/if}
        </section>

        <!-- CPU / Memory / Disk -->
        <div class="mets">
          {#each [
            { name: 'CPU', v: cpuPct, s: series.cpu, sub: $pulseMetrics?.metrics_available ? `${cores($pulseMetrics.cpu_usage ?? 0)} / ${cores($pulseMetrics.cpu_allocatable)} cores allocatable` : 'metrics-server not available' },
            { name: 'Memory', v: memPct, s: series.mem, sub: $pulseMetrics?.metrics_available ? `${gib($pulseMetrics.memory_usage ?? 0)} / ${gib($pulseMetrics.memory_allocatable)} Gi allocatable` : 'metrics-server not available' },
            { name: 'Disk', v: diskPct, s: series.disk, sub: worstDiskNode ? `worst node · ${worstDiskNode}` : 'kubelet stats not available' },
          ] as m (m.name)}
            <section class="met" aria-label="{m.name} {m.v === null ? 'unavailable' : `${Math.round(m.v)} percent`}">
              <div class="ml"><span>{m.name}</span><span class="tab">last 1h</span></div>
              <div class="mv" class:cf={m.v !== null && m.v >= 90} class:cw={m.v !== null && m.v >= 75 && m.v < 90}>
                {m.v === null ? '–' : `${Math.round(m.v)}%`}
                {#if m.v !== null && m.v >= 90}<StatusIcon health="failed" size={14} colored={false} />{:else if m.v !== null && m.v >= 75}<StatusIcon health="warning" size={14} colored={false} />{/if}
              </div>
              <LineSpark values={vals(m.s)} fixed />
              <div class="ms">{m.sub}</div>
            </section>
          {/each}
        </div>
      </div>

      <!-- Activity -->
      <section class="box actf" class:open={actOpen} aria-label="Activity">
        <div class="bh">
          <h2>Activity</h2>
          <span class="cnt">last hour · {$activity.length}</span>
          <button type="button" class="acttg" aria-expanded={actOpen} onclick={() => (actOpen = !actOpen)}>{actOpen ? 'Hide' : 'Show'}</button>
        </div>
        {#if $activity[0]}<div class="act-latest">Latest: {$activity[0].text} · {ago($activity[0].t)}</div>{/if}
        {#if $activity.length}
          <ul class="actl">
            {#each $activity as a (a.id)}
              {@const ic = actIcon(a)}
              <li class:on={triggerIds.has(a.id)}>
                <span class="ai {ic.cls}">
                  {#if ic.status}<StatusIcon health={ic.status} size={13} colored={false} />{:else if ic.icon}<Icon name={ic.icon} size={13} />{/if}
                </span>
                <div class="at">
                  <div>{a.text}</div>
                  <div class="ad">{a.detail}{a.by ? ` · by ${a.by}` : ''}</div>
                </div>
                <span class="atime">{ago(a.t)}</span>
              </li>
            {/each}
          </ul>
        {:else}
          <div class="act-empty">No changes in the last hour. Rollouts, scaling, restarts, node changes and Warning events show up here.</div>
        {/if}
      </section>
    </div>

    <!-- Issues -->
    <section class="box" aria-label="Issues">
      <div class="bh">
        <h2>Issues</h2>
        <span class="cnt">{issues.length}</span>
        <div class="r"><span class="muted hide-s" style="font-size:12px">j/k move · Enter details · Esc close</span></div>
      </div>
      {#if !issues.length}
        <div class="agood">
          <StatusIcon health="healthy" size={20} />
          <div><b>No issues on {$currentContext}.</b> Every workload is available and every node is ready.</div>
        </div>
      {:else}
        <div class="itab">
          <table>
            <thead>
              <tr><th>Sev</th><th>Object</th><th>Reason</th><th>Impact</th><th>Age</th><th class="acol"><span class="sr-only">Action</span></th></tr>
            </thead>
            <tbody>
              {#each issues as i, idx (i.id)}
                {@const sel = idx === $selectedRowIndex}
                {@const hint = triggerShort(i)}
                {@const logPod = logsTarget(i)}
                <tr
                  bind:this={rowEls[idx]}
                  class:sel
                  aria-selected={sel}
                  onclick={() => openIssue(i)}
                >
                  <td><span class="sevc {i.sev}"><StatusIcon health={i.sev === 'warn' ? 'warning' : 'failed'} colored={false} />{SEV_LABEL[i.sev]}</span></td>
                  <td class="ob"><span class="k">{KIND_SHORT[i.kind]}/</span>{i.name}{#if i.namespace}<span class="k"> · {i.namespace}</span>{/if}</td>
                  <td><StatusPill health={toDisplay(i.verdict).health} label={i.reason} variant="dot" /></td>
                  <td>{impactOf(i)}{#if hint}<span class="hint"> · {hint}</span>{/if}</td>
                  <td class="tab">{shortDuration(now - i.startedAt)}</td>
                  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
                  <td class="acol" onclick={(e) => e.stopPropagation()}>
                    <span class="ac">
                      {#if logPod}
                        <button type="button" class="btn" onclick={() => openPod(logPod, 'logs')}>Logs</button>
                      {:else}
                        <button type="button" class="btn" onclick={() => openObject(i)}>Open</button>
                      {/if}
                      <button type="button" class="ib" aria-label="Details for {i.name}" title="Details · Enter" onclick={() => openIssue(i)}><Icon name="more" /></button>
                    </span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </section>
  </div>

  {#if sheetOpen && selected}
    {@const logPod = logsTarget(selected)}
    <IssueSheet
      issue={selected}
      impact={impactOf(selected)}
      pods={podsOf(selected)}
      events={eventsFor(selected, $clusterEvents)}
      trigger={triggerOf(selected)}
      prod={$currentContextIsProd}
      {now}
      canRestart={canRestart(selected)}
      canScale={canRestart(selected)}
      onClose={() => (sheetOpen = false)}
      onLogs={logPod ? () => openPod(logPod, 'logs') : undefined}
      onOpen={() => openObject(selected)}
      onOpenPod={(p) => openPod(p)}
      onRestart={() => askRestart(selected)}
      onScale={() => askScale(selected)}
      onCopy={() => copyRef(selected)}
    />
  {/if}
</div>

{#if tip}
  <div class="pulse-tip" style="left:{tip.x}px;top:{tip.y}px;transform:translate(-50%,{tip.below ? '0' : '-100%'})">
    <div class="tn">{KIND_SHORT[tip.obj.kind]}/{tip.obj.name}{tip.obj.namespace ? ` · ${tip.obj.namespace}` : ''}</div>
    <StatusPill health={toDisplay(tip.obj.verdict).health} label={tip.obj.verdict.reason || 'Healthy'} variant="dot" />
  </div>
{/if}

{#if confirm}
  {@const c = confirm}
  <ConfirmDialog
    open={true}
    title={c.mode === 'restart' ? `Restart ${KIND_SHORT[c.issue.kind]}/${c.issue.name}?` : `Scale ${KIND_SHORT[c.issue.kind]}/${c.issue.name}`}
    resource={{ kind: c.issue.kind === 'deployment' ? 'Deployment' : 'StatefulSet', name: c.issue.name, namespace: c.issue.namespace ?? undefined, context: $currentContext }}
    confirmLabel={c.mode === 'restart' ? 'Restart' : 'Scale'}
    body={c.mode === 'restart' ? 'Pods are replaced one by one (rolling restart). Running requests on old pods may be cut off.' : scaleBody}
    successMessage={c.mode === 'restart' ? `Restarting ${c.issue.name}` : `Scaling ${c.issue.name} to ${replicas}`}
    onConfirm={runConfirm}
    onClose={() => (confirm = null)}
  />
{/if}

{#snippet scaleBody()}
  <label class="block text-sm text-secondary">
    Replicas
    <input
      type="number"
      min="0"
      bind:value={replicas}
      class="mt-1.5 w-28 h-8 px-2.5 rounded-md bg-app border border-line-strong text-sm font-mono text-primary"
    />
  </label>
{/snippet}
