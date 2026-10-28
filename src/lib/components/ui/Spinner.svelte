<script lang="ts">
  let { size = 14, class: className = '' }: { size?: number; class?: string } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  class="animate-spin shrink-0 {className}"
  aria-hidden="true"
>
  <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-opacity="0.25" stroke-width="2" />
  <path d="M14 8a6 6 0 0 0-6-6" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
</svg>
