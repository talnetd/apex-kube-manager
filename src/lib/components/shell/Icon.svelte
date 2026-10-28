<script lang="ts">
  // Decorative shell icon. The label next to it (or the button's aria-label)
  // carries the meaning.
  import { ICONS, type IconName } from './icons';

  let { name, size = 16, class: className = '' }: { name: IconName; size?: number; class?: string } = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 16 16"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  class="shrink-0 {className}"
><path d={ICONS[name]} /></svg>
