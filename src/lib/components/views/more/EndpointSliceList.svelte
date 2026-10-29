<script lang="ts">
  import ResourceListView from '../ResourceListView.svelte';
  import StatusPill from '../../ui/StatusPill.svelte';
  import { endpointSlices, type EndpointSliceInfo } from '../../../stores/resources';
  import { HEALTH_SORT_ID, type Column } from '../../ui/data-table';
  import { HEALTH_RANK, type HealthResult } from '../../../utils/health';

  // No endpoints behind a service is neutral (it may be scaled to zero);
  // some unready is a warning; none ready out of several is failed.
  function health(s: EndpointSliceInfo): HealthResult {
    if (s.total === 0) return { health: 'neutral', label: 'No endpoints', rank: HEALTH_RANK.neutral };
    if (s.ready === 0) return { health: 'failed', label: `0/${s.total} ready`, rank: HEALTH_RANK.failed };
    if (s.ready < s.total) return { health: 'warning', label: `${s.ready}/${s.total} ready`, rank: HEALTH_RANK.warning };
    return { health: 'healthy', label: `${s.ready}/${s.total} ready`, rank: HEALTH_RANK.healthy };
  }

  const columns: Column<EndpointSliceInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%' },
    { id: 'service', label: 'Service', sortKey: 'service', width: '18%', mono: true },
    { id: 'ready', label: 'Endpoints', sortKey: (s) => health(s).rank, width: '130px', cell: readyCell },
    { id: 'address_type', label: 'Type', sortKey: 'address_type', width: '72px', hideBelow: 1000 },
    { id: 'ports', label: 'Ports', sortKey: 'ports', width: '18%', hideBelow: 900 },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet readyCell(s: EndpointSliceInfo)}
  {@const h = health(s)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ResourceListView
  title="Endpoints"
  subtitle="EndpointSlices"
  itemLabel="endpoint slices"
  viewId="endpointslices"
  store={endpointSlices}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  resourceType="endpointslice"
  {health}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
/>
