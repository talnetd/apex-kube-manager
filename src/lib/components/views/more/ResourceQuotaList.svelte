<script lang="ts">
  import ListView from '../../ui/ListView.svelte';
  import StatusPill from '../../ui/StatusPill.svelte';
  import { resourceQuotas, type ResourceQuotaInfo } from '../../../stores/resources';
  import { HEALTH_SORT_ID, type Column } from '../../ui/data-table';
  import { formatQuotaPct, quotaHealth, quotaUsage, type QuotaRow } from '../../../utils/resources';
  import type { HealthResult } from '../../../utils/health';

  // Parsing quantities per render is cheap, but cache per row object anyway:
  // the table asks for health several times per row.
  const cache = new WeakMap<ResourceQuotaInfo, { rows: QuotaRow[]; health: HealthResult }>();
  function facts(q: ResourceQuotaInfo) {
    let f = cache.get(q);
    if (!f) {
      const rows = quotaUsage(q.hard, q.used);
      f = { rows, health: quotaHealth(rows) };
      cache.set(q, f);
    }
    return f;
  }
  const health = (q: ResourceQuotaInfo) => facts(q).health;
  const summary = (q: ResourceQuotaInfo) =>
    facts(q)
      .rows.slice(0, 3)
      .map((r) => `${r.resource} ${r.pct === null ? `${r.used || '?'}/${r.hard}` : formatQuotaPct(r.pct)}`)
      .join(' · ');

  const columns: Column<ResourceQuotaInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '16%' },
    { id: 'status', label: 'Fullest', sortKey: (q) => health(q).rank, width: '200px', cell: statusCell },
    { id: 'usage', label: 'Usage', width: '34%', hideBelow: 900, cell: usageCell, title: summary },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(q: ResourceQuotaInfo)}
  {@const h = health(q)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}
{#snippet usageCell(q: ResourceQuotaInfo)}{summary(q) || '–'}{/snippet}

<ListView
  title="Resource Quotas"
  itemLabel="resource quotas"
  viewId="resourcequotas"
  source={resourceQuotas}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  resourceType="resourcequota"
  {health}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
/>
