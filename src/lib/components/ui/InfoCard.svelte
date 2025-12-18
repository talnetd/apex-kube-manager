<script lang="ts">
  interface Props {
    label: string;
    value: string | number | null | undefined;
    mono?: boolean;
  }

  let { label, value, mono = false }: Props = $props();
</script>

<div class="bg-bg-secondary rounded-lg p-4">
  <div class="text-xs text-text-muted uppercase tracking-wide mb-1">{label}</div>
  <div class="text-text-primary {mono ? 'font-mono text-sm' : ''}">
    {value ?? '-'}
  </div>
</div>
