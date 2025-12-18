<script lang="ts">
  interface Condition {
    type?: string;
    condition_type?: string;
    status: string;
    reason?: string | null;
    message?: string | null;
    last_transition_time?: string | null;
    last_update_time?: string | null;
  }

  interface Props {
    conditions: Condition[];
    title?: string;
  }

  let { conditions, title = 'Conditions' }: Props = $props();

  function formatTime(time: string | null | undefined): string {
    if (!time) return '-';
    const date = new Date(time);
    return date.toLocaleString();
  }

  function getConditionType(c: Condition): string {
    return c.type || c.condition_type || '-';
  }
</script>

<section>
  <h2 class="text-lg font-semibold text-text-primary mb-4">{title}</h2>
  <div class="bg-bg-secondary rounded-lg overflow-hidden">
    <table class="w-full">
      <thead>
        <tr class="text-left text-xs text-text-muted uppercase tracking-wide border-b border-border-subtle">
          <th class="px-4 py-3">Type</th>
          <th class="px-4 py-3">Status</th>
          <th class="px-4 py-3">Reason</th>
          <th class="px-4 py-3 hidden md:table-cell">Message</th>
          <th class="px-4 py-3">Last Transition</th>
        </tr>
      </thead>
      <tbody>
        {#each conditions as condition}
          <tr class="border-b border-border-subtle last:border-0 hover:bg-bg-tertiary/50">
            <td class="px-4 py-3 text-text-primary font-medium">{getConditionType(condition)}</td>
            <td class="px-4 py-3">
              <span class="inline-flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full {condition.status === 'True' ? 'bg-accent-success' : condition.status === 'False' ? 'bg-accent-error' : 'bg-accent-warning'}"></span>
                <span class="text-text-secondary">{condition.status}</span>
              </span>
            </td>
            <td class="px-4 py-3 text-text-secondary text-sm">{condition.reason || '-'}</td>
            <td class="px-4 py-3 text-text-muted text-sm hidden md:table-cell max-w-xs truncate" title={condition.message || ''}>
              {condition.message || '-'}
            </td>
            <td class="px-4 py-3 text-text-muted text-sm">
              {formatTime(condition.last_transition_time || condition.last_update_time)}
            </td>
          </tr>
        {:else}
          <tr>
            <td colspan="5" class="px-4 py-8 text-center text-text-muted">No conditions</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
