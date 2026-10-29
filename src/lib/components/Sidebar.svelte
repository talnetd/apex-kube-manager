<script lang="ts">
  // Sidebar (spec 4.2, 13.2; approved Geist preview). Sentence-case section
  // headers that collapse, active item as a neutral pill, mono problem counts
  // with no fill. md: icons only, counts become a dot. sm: off-canvas drawer.
  import { currentView, navOpen, type View } from '../stores/navigation';
  import { onMount } from 'svelte';
  import {
    pulseMetrics,
    pulseScope,
    clusterEvents,
    nodes,
    currentContext,
    selectedNamespace,
    loadPulseMetrics,
  } from '../stores/kubernetes';
  import { updateInfo } from '../stores/updates';
  import Icon from './shell/Icon.svelte';
  import type { IconName } from './shell/icons';

  interface NavItem {
    id: View;
    label: string;
    icon: IconName;
  }
  interface NavSection {
    id: string;
    title?: string;
    items: NavItem[];
  }

  const sections: NavSection[] = [
    {
      id: 'top',
      items: [
        { id: 'dashboard', label: 'Pulse', icon: 'pulse' },
        { id: 'events', label: 'Events', icon: 'events' },
      ],
    },
    {
      id: 'wl',
      title: 'Workloads',
      items: [
        { id: 'pods', label: 'Pods', icon: 'pods' },
        { id: 'deployments', label: 'Deployments', icon: 'deploy' },
        { id: 'statefulsets', label: 'StatefulSets', icon: 'sts' },
        { id: 'daemonsets', label: 'DaemonSets', icon: 'ds' },
        { id: 'replicasets', label: 'ReplicaSets', icon: 'rs' },
        { id: 'jobs', label: 'Jobs', icon: 'job' },
        { id: 'cronjobs', label: 'CronJobs', icon: 'cron' },
      ],
    },
    {
      id: 'net',
      title: 'Network',
      items: [
        { id: 'services', label: 'Services', icon: 'svc' },
        { id: 'ingresses', label: 'Ingresses', icon: 'ing' },
        { id: 'network_policies', label: 'Network Policies', icon: 'netpol' },
      ],
    },
    {
      id: 'cfg',
      title: 'Config',
      items: [
        { id: 'configmaps', label: 'ConfigMaps', icon: 'cm' },
        { id: 'secrets', label: 'Secrets', icon: 'secret' },
      ],
    },
    {
      id: 'sto',
      title: 'Storage',
      items: [
        { id: 'pvs', label: 'Persistent Volumes', icon: 'pv' },
        { id: 'pvcs', label: 'PV Claims', icon: 'pvc' },
      ],
    },
    { id: 'policy', title: 'Scaling & Policy', items: [{ id: 'hpas', label: 'HPAs', icon: 'hpa' }] },
    { id: 'access', title: 'Access Control', items: [{ id: 'serviceaccounts', label: 'Service Accounts', icon: 'sa' }] },
    {
      id: 'cl',
      title: 'Cluster',
      items: [
        { id: 'nodes', label: 'Nodes', icon: 'node' },
        { id: 'namespaces', label: 'Namespaces', icon: 'ns' },
      ],
    },
  ];

  // Collapsed sections persist per machine; a convenience, so storage errors are ignored.
  const COLLAPSE_KEY = 'apex-sidebar-collapsed';
  function readCollapsed(): Record<string, boolean> {
    try {
      return JSON.parse(localStorage.getItem(COLLAPSE_KEY) ?? '{}') ?? {};
    } catch {
      return {};
    }
  }
  let collapsed = $state<Record<string, boolean>>(readCollapsed());
  $effect(() => {
    try {
      localStorage.setItem(COLLAPSE_KEY, JSON.stringify(collapsed));
    } catch {
      /* private mode */
    }
  });

  const titled = sections.filter((s) => s.title);
  const anyOpen = $derived(titled.some((s) => !collapsed[s.id]));
  function toggleAll() {
    const next = anyOpen;
    collapsed = Object.fromEntries(titled.map((s) => [s.id, next]));
  }

  // Workload badges read the Pulse metrics. Keep them for the current
  // context and namespace: reload on change and refresh every 30s (Pulse
  // itself polls every 5s while it's open).
  $effect(() => {
    const ctx = $currentContext;
    const ns = $selectedNamespace;
    if (ctx) loadPulseMetrics(ns);
  });
  onMount(() => {
    const id = setInterval(() => loadPulseMetrics($selectedNamespace), 30000);
    return () => clearInterval(id);
  });
  const metricsMatch = $derived(
    !!$pulseScope && $pulseScope.context === $currentContext && $pulseScope.namespace === ($selectedNamespace ?? null)
  );

  // sm drawer: Esc closes it.
  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && $navOpen) {
      e.preventDefault();
      navOpen.set(false);
    }
  }

  function go(view: View) {
    currentView.set(view);
    navOpen.set(false); // also when the view didn't change
  }

  type Badge = { tone: 'fail' | 'warn'; n: number };

  // Problem counts from data the app already polls: the Pulse metrics
  // (triage verdicts per workload kind), the event watch and the node watch.
  const badges = $derived.by(() => {
    const b: Partial<Record<View, Badge>> = {};
    const m = $pulseMetrics;
    if (m && metricsMatch) {
      for (const k of ['pods', 'deployments', 'statefulsets', 'daemonsets', 'replicasets', 'jobs'] as const) {
        if (m[k].fail > 0) b[k] = { tone: 'fail', n: m[k].fail };
      }
    }
    const warnEvents = $clusterEvents.filter((e) => e.type === 'Warning').length;
    if (warnEvents > 0) b.events = { tone: 'warn', n: warnEvents };
    const badNodes = $nodes.filter((n) => n.verdict.status === 'fail' || n.verdict.status === 'warn');
    if (badNodes.length) {
      b.nodes = { tone: badNodes.some((n) => n.verdict.status === 'fail') ? 'fail' : 'warn', n: badNodes.length };
    }
    return b;
  });

  function sectionBadge(sec: NavSection): Badge | null {
    let n = 0;
    let fail = false;
    for (const i of sec.items) {
      const x = badges[i.id];
      if (!x) continue;
      n += x.n;
      fail ||= x.tone === 'fail';
    }
    return n ? { tone: fail ? 'fail' : 'warn', n } : null;
  }

  const fmt = (n: number) => (n > 99 ? '99+' : String(n));
  const ariaFor = (i: NavItem, x?: Badge) =>
    i.label + (x ? `, ${x.n} ${x.tone === 'fail' ? 'failing' : 'warnings'}` : '');
</script>

{#snippet badge(x: Badge | null | undefined)}
  {#if x}
    <span class="badge {x.tone === 'fail' ? 'text-danger b-fail' : 'text-warning b-warn'}" aria-hidden="true"
      >{x.tone === 'warn' ? '▲' : ''}{fmt(x.n)}</span
    >
  {/if}
{/snippet}

<svelte:window onkeydown={onKeydown} />

<!-- sm: scrim behind the drawer -->
{#if $navOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="navscrim fixed inset-0 z-[35] bg-scrim" onclick={() => navOpen.set(false)}></div>
{/if}

<aside class="side" class:open={$navOpen} aria-label="Navigation">
  <div class="brand" data-tauri-drag-region>
    <span class="logo" aria-hidden="true" data-tauri-drag-region></span>
    <span class="brand-name" data-tauri-drag-region>Apex</span>
  </div>

  <nav class="nav">
    {#each sections as sec (sec.id)}
      {@const isCollapsed = !!(sec.title && collapsed[sec.id])}
      {#if sec.title}
        <div class="rail-div" aria-hidden="true"></div>
        <button
          type="button"
          class="sect"
          aria-expanded={!isCollapsed}
          onclick={() => (collapsed = { ...collapsed, [sec.id]: !collapsed[sec.id] })}
        >
          <svg class="chev" width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"
            ><path d="M2.5 3.5L5 6l2.5-2.5" /></svg
          >
          <span class="sect-title">{sec.title}</span>
          {#if isCollapsed}{@render badge(sectionBadge(sec))}{/if}
        </button>
      {/if}
      <!-- Collapsed items stay in the DOM: the md icon rail always shows them. -->
      {#each sec.items as item (item.id)}
          {@const x = badges[item.id]}
          <button
            type="button"
            class="item"
            class:folded={isCollapsed}
            title={item.label}
            aria-label={ariaFor(item, x)}
            aria-current={$currentView === item.id ? 'page' : undefined}
            onclick={() => go(item.id)}
          >
            <Icon name={item.icon} />
            <span class="lbl">{item.label}</span>
            {@render badge(x)}
          </button>
      {/each}
    {/each}
  </nav>

  <div class="sidefoot">
    <span class="font-mono">{$updateInfo?.current_version ? `v${$updateInfo.current_version}` : ''}</span>
    <button type="button" class="hover:text-primary" title="Collapse or expand all sections" onclick={toggleAll}
      >{anyOpen ? 'Collapse all' : 'Expand all'}</button
    >
  </div>
</aside>

<style>
  .side {
    grid-row: 1 / 3;
    grid-column: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    background: var(--ui-app);
    border-right: 1px solid var(--ui-line);
    z-index: 40;
  }
  .brand {
    height: 44px;
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    border-bottom: 1px solid var(--ui-line);
    font-weight: 600;
    letter-spacing: -0.01em;
    user-select: none;
  }
  /* Geist mark: a plain triangle in the primary text colour. */
  .logo {
    width: 0;
    height: 0;
    border-left: 7px solid transparent;
    border-right: 7px solid transparent;
    border-bottom: 13px solid var(--ui-primary);
  }
  .nav {
    flex: 1;
    overflow: auto;
    padding: 6px 8px 12px;
  }
  .sect {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 18px 6px 4px;
    font-size: 13px;
    line-height: 16px;
    font-weight: 500;
    color: var(--ui-primary);
    border-radius: 6px;
    text-align: left;
  }
  .sect .chev {
    flex: none;
    color: var(--ui-muted);
    transition: transform 0.15s;
  }
  .sect[aria-expanded='false'] .chev {
    transform: rotate(-90deg);
  }
  .sect .badge {
    margin-left: auto;
  }
  .item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    height: var(--nav-item-h);
    padding: 0 8px;
    border-radius: 6px;
    color: var(--ui-secondary);
    font-size: var(--nav-font);
    text-align: left;
  }
  .item :global(svg) {
    color: var(--ui-faint);
  }
  .item.folded {
    display: none;
  }
  .item:hover {
    background: var(--ui-hover);
    color: var(--ui-primary);
  }
  .item[aria-current='page'] {
    background: var(--ui-active);
    color: var(--ui-primary);
    font-weight: 500;
  }
  .item[aria-current='page'] :global(svg) {
    color: var(--ui-secondary);
  }
  .lbl {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .sidefoot {
    flex: none;
    display: flex;
    justify-content: space-between;
    gap: 8px;
    padding: 8px 12px;
    border-top: 1px solid var(--ui-line);
    font-size: 12px;
    color: var(--ui-muted);
  }
  .navscrim {
    display: none;
  }
  .rail-div {
    display: none;
  }

  /* md 640-1199: icons only, 52px; counts become a dot on the icon. */
  @media (min-width: 640px) and (max-width: 1199.98px) {
    .brand {
      justify-content: center;
      padding: 0;
    }
    .brand-name,
    .lbl,
    .sidefoot {
      display: none;
    }
    .nav {
      padding: 6px;
    }
    .sect {
      display: none;
    }
    .rail-div {
      display: block;
      height: 1px;
      margin: 6px 4px;
      background: var(--ui-divider);
    }
    .item,
    .item.folded {
      display: flex;
      justify-content: center;
      padding: 0;
    }
    .item .badge {
      position: absolute;
      top: 4px;
      right: 6px;
      width: 7px;
      height: 7px;
      padding: 0;
      font-size: 0;
      border-radius: 50%;
    }
    .item .b-fail {
      background: var(--ui-danger-solid);
    }
    .item .b-warn {
      background: var(--ui-warning-solid);
    }
  }

  /* sm < 640: off-canvas drawer opened from the header's menu button. */
  @media (max-width: 639.98px) {
    .side {
      position: fixed;
      left: 0;
      top: 0;
      bottom: 24px;
      width: 260px;
      transform: translateX(-102%);
      visibility: hidden;
      transition:
        transform 0.2s cubic-bezier(0.2, 0, 0, 1),
        visibility 0s linear 0.2s;
    }
    .side.open {
      transform: none;
      visibility: visible;
      transition: transform 0.2s cubic-bezier(0.2, 0, 0, 1);
      box-shadow: var(--ui-shadow-dialog);
    }
    .navscrim {
      display: block;
    }
  }
</style>
