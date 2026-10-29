<script lang="ts">
  import ListView from '../ui/ListView.svelte';
  import { BP, type Column } from '../ui/data-table';
  import {
    serviceList,
    type ServiceInfo,
  } from '../../stores/kubernetes';

  const columns: Column<ServiceInfo>[] = [
    { id: 'name', label: 'Name', sortKey: 'name', primary: true },
    { id: 'namespace', label: 'Namespace', sortKey: 'namespace', width: '14%', hideBelow: BP.md },
    { id: 'service_type', label: 'Type', sortKey: 'service_type', width: '110px' },
    { id: 'cluster_ip', label: 'Cluster IP', sortKey: 'cluster_ip', mono: true, width: '128px', hideBelow: BP.wide },
    { id: 'external_ip', label: 'External IP', sortKey: 'external_ip', mono: true, width: '16%', hideBelow: BP.lg },
    { id: 'ports', label: 'Ports', mono: true, width: '18%', hideBelow: BP.md },
    { id: 'age', label: 'Age', sortKey: 'age', numeric: true, width: '64px' },
  ];
</script>

<ListView
  title="Services"
  viewId="services"
  itemLabel="services"
  source={serviceList}
  {columns}
  rowKey={(s) => `${s.namespace}/${s.name}`}
  filterFields={['name', 'namespace', 'service_type', 'cluster_ip', 'external_ip', 'ports']}
  defaultSort={{ id: 'name', dir: 'asc' }}
  resourceType="service"
/>
