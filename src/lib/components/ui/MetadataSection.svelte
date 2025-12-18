<script lang="ts">
  import KeyValueGrid from './KeyValueGrid.svelte';

  interface Props {
    labels: Record<string, string>;
    annotations: Record<string, string>;
  }

  let { labels, annotations }: Props = $props();
</script>

<section class="grid grid-cols-1 md:grid-cols-2 gap-6">
  <div>
    <h2 class="text-lg font-semibold text-text-primary mb-4">Labels</h2>
    <KeyValueGrid data={labels} emptyText="No labels" />
  </div>
  <div>
    <h2 class="text-lg font-semibold text-text-primary mb-4">Annotations</h2>
    <KeyValueGrid data={annotations} emptyText="No annotations" maxHeight="12rem" />
  </div>
</section>
