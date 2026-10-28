<script lang="ts">
  // Feeds a writable store into DataTable so tests can reorder rows the way
  // a watch stream does.
  import type { Writable } from 'svelte/store';
  import DataTable from '../../src/lib/components/ui/DataTable.svelte';
  import type { Column, TableSort } from '../../src/lib/components/ui/data-table';
  import type { HealthResult } from '../../src/lib/utils/health';

  interface Row {
    name: string;
    cpu: number;
    rank?: number;
  }

  let {
    rows,
    onOpen,
    defaultSort,
  }: {
    rows: Writable<Row[]>;
    onOpen: (r: Row) => void;
    defaultSort?: TableSort;
  } = $props();

  const columns: Column<Row>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'cpu', label: 'CPU', sortKey: 'cpu', numeric: true },
  ];

  const health = (r: Row): HealthResult =>
    r.rank === 0
      ? { health: 'failed', label: 'CrashLoopBackOff', rank: 0 }
      : { health: 'healthy', label: 'Running', rank: 4 };
</script>

<DataTable
  rows={$rows}
  {columns}
  rowKey={(r) => r.name}
  {health}
  {defaultSort}
  state="ready"
  itemLabel="pods"
  {onOpen}
/>
