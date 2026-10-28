<script lang="ts">
  // One status shape (spec 11.1). Decorative by default: the label next to it
  // carries the meaning. Pass `label` when the icon stands alone.
  import { STATUS_ICONS, STATUS_ICON_VIEWBOX } from './status-icons';
  import { HEALTH_CLASSES, type Health } from '../../utils/health';

  let {
    health,
    size = 12,
    label,
    class: className = '',
    colored = true,
  }: {
    health: Health;
    size?: number;
    label?: string;
    class?: string;
    /** false = inherit currentColor from the parent. */
    colored?: boolean;
  } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox={STATUS_ICON_VIEWBOX}
  fill="none"
  stroke="currentColor"
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class="shrink-0 {colored ? HEALTH_CLASSES[health].icon : ''} {className}"
  role={label ? 'img' : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : 'true'}
>
  {#each STATUS_ICONS[health] as p}
    {#if p.fill}
      <path d={p.d} fill="currentColor" stroke="none" />
    {:else}
      <path d={p.d} />
    {/if}
  {/each}
</svg>
