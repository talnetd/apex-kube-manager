<script lang="ts">
  import ListView from '../../ui/ListView.svelte';
  import StatusPill from '../../ui/StatusPill.svelte';
  import { pdbs, type PdbInfo } from '../../../stores/resources';
  import { HEALTH_SORT_ID, type Column } from '../../ui/data-table';
  import { pdbHealth } from '../../../utils/resources';

  const columns: Column<PdbInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '16%' },
    { id: 'status', label: 'Status', sortKey: (p) => pdbHealth(p).rank, width: '170px', cell: statusCell },
    { id: 'min_available', label: 'Min available', sortKey: 'min_available', numeric: true, width: '110px', hideBelow: 900 },
    { id: 'max_unavailable', label: 'Max unavailable', sortKey: 'max_unavailable', numeric: true, width: '130px', hideBelow: 900 },
    { id: 'allowed_disruptions', label: 'Allowed', sortKey: 'allowed_disruptions', numeric: true, width: '80px' },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

{#snippet statusCell(p: PdbInfo)}
  {@const h = pdbHealth(p)}
  <StatusPill health={h.health} label={h.label} />
{/snippet}

<ListView
  title="Pod Disruption Budgets"
  itemLabel="pod disruption budgets"
  viewId="pdbs"
  source={pdbs}
  {columns}
  rowKey={(r) => `${r.namespace}/${r.name}`}
  resourceType="pdb"
  health={pdbHealth}
  defaultSort={{ id: HEALTH_SORT_ID, dir: 'asc' }}
  emptyHint="Budgets limit how many pods voluntary disruptions (drains, rollouts) can take down at once."
/>
