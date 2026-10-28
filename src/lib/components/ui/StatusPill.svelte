<script lang="ts">
  // Status display (spec 5.2, 11.1-11.3): icon shape + text label + colour,
  // never colour alone. Default variant follows density through CSS
  // (comfortable = soft, compact = dot), so no JS branches on density.
  import StatusIcon from './StatusIcon.svelte';
  import { HEALTH_CLASSES, healthAriaLabel, type Health } from '../../utils/health';

  let {
    health,
    label,
    variant,
    title,
  }: {
    health: Health;
    label: string;
    variant?: 'soft' | 'dot';
    /** Tooltip, e.g. the full container message. Defaults to the label when it may truncate. */
    title?: string;
  } = $props();

  const cls = $derived(HEALTH_CLASSES[health]);
  const mode = $derived(variant ?? 'auto');
  const tip = $derived(title ?? (label.length > 24 ? label : undefined));
</script>

<span
  class="pill inline-flex items-center gap-1 max-w-full font-medium whitespace-nowrap {cls.fg} {mode === 'dot' ? '' : cls.tint}"
  class:soft={mode === 'soft'}
  class:auto={mode === 'auto'}
  class:tinted={cls.tint !== ''}
  title={tip}
  aria-label={healthAriaLabel(health, label)}
>
  <StatusIcon {health} colored={false} />
  <span class="truncate max-w-[24ch]" aria-hidden="true">{label}</span>
</span>

<style>
  .pill {
    font-size: 12px;
    line-height: 16px;
  }
  .soft,
  .auto {
    height: var(--badge-h);
    border-radius: var(--radius-sm);
  }
  .soft.tinted,
  .auto.tinted {
    padding-inline: 6px;
  }
  /* Compact: the auto variant drops to dot (no tint), text-2xs. */
  :global([data-density='compact']) .auto {
    background: transparent;
    padding-inline: 0;
    font-size: 11px;
  }
  :global([data-density='compact']) .soft {
    font-size: 11px;
  }
</style>
