<script lang="ts">
  interface Props {
    status: string;
  }

  let { status }: Props = $props();

  function getStatusVariant(s: string): 'success' | 'warning' | 'error' | 'default' {
    const lower = s.toLowerCase();
    if (lower === 'running' || lower === 'succeeded' || lower === 'active' || lower === 'ready') {
      return 'success';
    }
    if (lower === 'pending' || lower === 'waiting' || lower === 'containercreating') {
      return 'warning';
    }
    if (lower === 'failed' || lower === 'error' || lower === 'crashloopbackoff' || lower === 'imagepullbackoff') {
      return 'error';
    }
    return 'default';
  }

  const variantStyles = {
    success: 'bg-accent-success/20 text-accent-success',
    warning: 'bg-accent-warning/20 text-accent-warning',
    error: 'bg-accent-error/20 text-accent-error',
    default: 'bg-text-muted/20 text-text-secondary',
  };

  let variant = $derived(getStatusVariant(status));
</script>

<span class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium {variantStyles[variant]}">
  <span class="w-1.5 h-1.5 rounded-full bg-current"></span>
  {status}
</span>
