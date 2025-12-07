<script lang="ts">
  interface Props {
    variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
    size?: 'sm' | 'md';
  }

  let { variant = 'default', size = 'md' }: Props = $props();

  const variantStyles = {
    default: 'bg-text-muted/20 text-text-secondary',
    success: 'bg-accent-success/20 text-accent-success',
    warning: 'bg-accent-warning/20 text-accent-warning',
    error: 'bg-accent-error/20 text-accent-error',
    info: 'bg-accent-primary/20 text-accent-primary',
  };

  const sizeStyles = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-3 py-1 text-sm',
  };
</script>

<span class="inline-flex items-center rounded-full font-medium {variantStyles[variant]} {sizeStyles[size]}">
  <slot />
</span>
