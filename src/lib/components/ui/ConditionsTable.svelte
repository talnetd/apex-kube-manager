<script lang="ts">
  import StatusPill from './StatusPill.svelte';
  import { conditionHealth } from '../../utils/conditions';
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
    if (!time) return '–';
    const date = new Date(time);
    return date.toLocaleString();
  }

  function getConditionType(c: Condition): string {
    return c.type || c.condition_type || '-';
  }
</script>

<section>
  <h2 class="d-sec-title">{title}</h2>
  <div class="d-frame">
    <table class="d-table">
      <thead>
        <tr>
          <th>Type</th>
          <th>Status</th>
          <th>Reason</th>
          <th class="hidden md:table-cell">Message</th>
          <th class="hidden sm:table-cell">Last transition</th>
        </tr>
      </thead>
      <tbody>
        {#each conditions as condition}
          <tr>
            <td>{getConditionType(condition)}</td>
            <td><StatusPill health={conditionHealth(getConditionType(condition), condition.status)} label={condition.status} variant="dot" /></td>
            <td>{condition.reason || '–'}</td>
            <td class="hidden md:table-cell max-w-xs truncate" title={condition.message || ''}>{condition.message || '–'}</td>
            <td class="hidden sm:table-cell tabular-nums text-muted">{formatTime(condition.last_transition_time || condition.last_update_time)}</td>
          </tr>
        {:else}
          <tr><td colspan="5" class="text-center text-muted py-6">No conditions</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
