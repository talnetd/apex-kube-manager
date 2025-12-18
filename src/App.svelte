<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Header from './lib/components/Header.svelte';
  import MainPanel from './lib/components/MainPanel.svelte';
  import StartupScreen from './lib/components/StartupScreen.svelte';
  import GlobalSearch from './lib/components/GlobalSearch.svelte';
  import { currentView } from './lib/stores/navigation';
  import { isInitialized } from './lib/stores/startup';
  import { globalSearchOpen } from './lib/stores/search';
  import { portForwardPanelOpen } from './lib/stores/portforward';

  function handleKeydown(e: KeyboardEvent) {
    // Cmd+K (Mac) or Ctrl+K (Windows/Linux) to open global search
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      globalSearchOpen.set(true);
    }
    // Cmd+P (Mac) or Ctrl+P (Windows/Linux) to open port forwards
    if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
      e.preventDefault();
      portForwardPanelOpen.update(v => !v);
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if !$isInitialized}
  <StartupScreen />
{:else}
  <div class="flex h-full bg-bg-primary">
    <!-- Sidebar -->
    <Sidebar />

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- Header -->
      <Header />

      <!-- Main Panel -->
      <MainPanel view={$currentView} />
    </div>
  </div>

  <!-- Global Search Modal -->
  <GlobalSearch />
{/if}
