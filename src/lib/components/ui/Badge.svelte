<script lang="ts">
  interface Props {
    variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
    size?: 'sm' | 'md';
  }

  let { variant = 'default', size = 'md' }: Props = $props();

  const variantStyles = {
    default: 'bg-text-muted/20 text-text-secondary',
    success: 'bg-accent-success/20 text-success',
    warning: 'bg-accent-warning/20 text-warning',
    error: 'bg-accent-error/20 text-danger',
    info: 'bg-accent-primary/20 text-accent-text',
  };

  const sizeStyles = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-3 py-1 text-base',
  };
</script>

<span class="inline-flex items-center rounded-full font-medium {variantStyles[variant]} {sizeStyles[size]}">
  <slot />
</span>
