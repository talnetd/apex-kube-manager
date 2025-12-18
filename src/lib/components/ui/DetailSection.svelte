<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    children: Snippet;
  }

  let { title, children }: Props = $props();
</script>

<section>
  <h2 class="text-lg font-semibold text-text-primary mb-4">{title}</h2>
  {@render children()}
</section>
