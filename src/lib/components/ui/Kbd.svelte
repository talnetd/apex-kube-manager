<script lang="ts">
  // Keyboard hint (spec 5.7).
  let { keys, class: className = '' }: { keys: string; class?: string } = $props();
</script>

<kbd
  class="inline-flex items-center h-4 px-1 rounded-sm border border-line-strong bg-transparent text-muted font-mono text-2xs leading-none whitespace-nowrap {className}"
>{keys}</kbd>
