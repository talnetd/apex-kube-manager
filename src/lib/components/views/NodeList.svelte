<script lang="ts">
  // Nodes: watched rows + polled usage (CLAUDE.md, "Node metrics sources").
  // The in-flight and context guards live in `loadNodeMetrics` (store), and
  // DataTable anchors the selection by node name (rowKey) after every
  // re-sort, so the sortable usage columns can change each poll without the
  // highlight drifting onto another node. Cordon and taints stay in the
  // node detail window.
  import { onMount, onDestroy } from 'svelte';
  import ListView from '../ui/ListView.svelte';
  import StatusPill from '../ui/StatusPill.svelte';
  import UsageBar from '../ui/UsageBar.svelte';
  import { BP, HEALTH_SORT_ID, type Column } from '../ui/data-table';
  import { verdictHealth } from '../../utils/status-words';
  import {
    nodes,
    nodeMetrics,
    currentContext,
    refreshTrigger,
    startNodeWatch,
    stopNodeWatch,
    loadNodeMetrics,
    nodeList,
    type NodeInfo,
    type ListSource,
  } from '../../stores/kubernetes';

  const METRICS_INTERVAL = 10000;

  type NodeRow = NodeInfo & {
    cpu_pct: number | null;
    memory_pct: number | null;
    disk_pct: number | null;
    cpu_detail: string;
    memory_detail: string;
    disk_detail: string;
  };

  function percent(used: number | null | undefined, total: number | null | undefined): number | null {
    if (used == null || !total) return null;
    return (used / total) * 100;
  }

  function formatCpu(millicores: number): string {
    if (millicores >= 1000) return `${(millicores / 1000).toFixed(2)} cores`;
    return `${Math.round(millicores)}m`;
  }

  function formatBytes(bytes: number): string {
    const gib = bytes / 1024 ** 3;
    if (gib >= 1) return `${gib.toFixed(1)} Gi`;
    return `${(bytes / 1024 ** 2).toFixed(0)} Mi`;
  }

  // Usage lives in a separate (polled) store; merge it onto the watched rows
  // so the percentages are sortable columns.
  const rows = $derived.by<NodeRow[]>(() => {
    const metrics = $nodeMetrics;
    return $nodes.map((node) => {
      const m = metrics[node.name];
      return {
        ...node,
        cpu_pct: percent(m?.cpu_usage, node.cpu_allocatable),
        memory_pct: percent(m?.memory_usage, node.memory_allocatable),
        disk_pct: percent(m?.disk_usage, m?.disk_capacity),
        cpu_detail: m?.cpu_usage != null ? `${formatCpu(m.cpu_usage)} / ${formatCpu(node.cpu_allocatable)}` : '',
        memory_detail: m?.memory_usage != null ? `${formatBytes(m.memory_usage)} / ${formatBytes(node.memory_allocatable)}` : '',
        disk_detail: m?.disk_usage != null && m?.disk_capacity != null ? `${formatBytes(m.disk_usage)} / ${formatBytes(m.disk_capacity)}` : '',
      };
    });
  });

  // Rows are rebuilt each metrics poll, so cache by the watched NodeInfo
  // object (stable until the node itself changes), not by the merged row.
  const nodeHealth = verdictHealth<NodeInfo>();
  const byName = $derived(new Map($nodes.map((n) => [n.name, n])));
  const health = (r: NodeRow) => nodeHealth(byName.get(r.name) ?? r);

  // The table shows `rows` (nodes merged with usage); the source still
  // supplies load state and the context the nodes came from.
  const nodeSource = nodeList as unknown as ListSource<NodeRow>;

  const taintText = (n: NodeRow) => n.taints.map((t) => `${t.key}${t.value ? `=${t.value}` : ''}:${t.effect}`).join(', ');

  const columns: Column<NodeRow>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    {
      id: 'status',
      label: 'Status',
      width: '15%',
      sortKey: (n) => health(n).rank,
      cell: statusCell,
      title: (n) => `${n.status} · ${n.verdict.reason} (${n.verdict.ruleId})`,
    },
    { id: 'roles', label: 'Roles', sortKey: (n) => n.roles.join(','), width: '10%', hideBelow: BP.wide },
    { id: 'cpu_pct', label: 'CPU', sortKey: 'cpu_pct', width: '108px', cell: cpuCell },
    { id: 'memory_pct', label: 'Memory', sortKey: 'memory_pct', width: '108px', cell: memoryCell },
    { id: 'disk_pct', label: 'Disk', sortKey: 'disk_pct', width: '108px', hideBelow: BP.wide, cell: diskCell },
    { id: 'version', label: 'Version', sortKey: 'version', mono: true, width: '10%', hideBelow: BP.lg },
    { id: 'internal_ip', label: 'Internal IP', sortKey: 'internal_ip', mono: true, width: '112px', hideBelow: BP.lg },
    { id: 'os', label: 'OS / Runtime', sortKey: 'os_image', width: '14%', hideBelow: BP.lg, cell: osCell, title: (n) => `${n.os_image} · ${n.container_runtime}` },
    { id: 'taints', label: 'Taints', sortKey: (n) => n.taints.length, numeric: true, width: '64px', hideBelow: BP.lg, title: (n) => taintText(n) || undefined, cell: taintsCell },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];

  // Metrics need the watched node names, so wait for the watch's first list.
  const refresh = () => startNodeWatch().then(() => loadNodeMetrics());

  let metricsTimer: ReturnType<typeof setInterval> | null = null;
  onMount(() => {
    metricsTimer = setInterval(loadNodeMetrics, METRICS_INTERVAL);
  });

  onDestroy(() => {
    stopNodeWatch();
    if (metricsTimer) clearInterval(metricsTimer);
  });

  $effect(() => {
    const ctx = $currentContext;
    void $refreshTrigger;
    if (!ctx) return;
    refresh();
  });
</script>

{#snippet statusCell(n: NodeRow)}
  {@const h = health(n)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

{#snippet cpuCell(n: NodeRow)}
  <UsageBar percent={n.cpu_pct} detail={n.cpu_detail} unavailableHint="No CPU usage: metrics-server not available" />
{/snippet}

{#snippet memoryCell(n: NodeRow)}
  <UsageBar percent={n.memory_pct} detail={n.memory_detail} unavailableHint="No memory usage: metrics-server not available" />
{/snippet}

{#snippet diskCell(n: NodeRow)}
  <UsageBar percent={n.disk_pct} detail={n.disk_detail} unavailableHint="No disk usage: kubelet stats unreachable (needs nodes/proxy access)" />
{/snippet}

{#snippet osCell(n: NodeRow)}{n.os_image}{/snippet}

{#snippet taintsCell(n: NodeRow)}{n.taints.length || '–'}{/snippet}

<ListView
  title="Nodes"
  viewId="nodes"
  itemLabel="nodes"
  subtitle="cluster-scoped"
  source={nodeSource}
  autoload={false}
  {rows}
  {columns}
  rowKey={(n) => n.name}
  filterFields={['name', 'status', 'roles', 'version', 'internal_ip', (n) => n.verdict.reason]}
  {health}
  segmentLabels={{ failed: 'Not ready', warning: 'Warning', healthy: 'Ready' }}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  resourceType="node"
  namespaced={false}
  onRefresh={refresh}
/>
