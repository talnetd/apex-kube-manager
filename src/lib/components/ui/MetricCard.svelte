<script lang="ts">
  interface Props {
    label: string;
    value: string | number;
    variant?: 'default' | 'success' | 'warning' | 'error';
    icon?: string;
  }

  let { label, value, variant = 'default', icon }: Props = $props();

  const variantColors = {
    default: 'text-text-primary',
    success: 'text-accent-success',
    warning: 'text-accent-warning',
    error: 'text-accent-error',
  };
</script>

<div class="bg-bg-card border border-border-subtle rounded-lg p-4">
  <div class="flex items-start justify-between">
    <div>
      <p class="text-sm text-text-muted uppercase tracking-wide mb-1">{label}</p>
      <p class="text-3xl font-semibold {variantColors[variant]}">{value}</p>
    </div>
    {#if icon}
      <div class="p-2 bg-bg-tertiary rounded-lg">
        <svg class="w-5 h-5 text-text-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={icon} />
        </svg>
      </div>
    {/if}
  </div>
</div>
