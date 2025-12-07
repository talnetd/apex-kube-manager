<script lang="ts">
  import Sidebar from './lib/components/Sidebar.svelte';
  import Header from './lib/components/Header.svelte';
  import MainPanel from './lib/components/MainPanel.svelte';
  import StartupScreen from './lib/components/StartupScreen.svelte';
  import { currentView } from './lib/stores/navigation';
  import { isInitialized } from './lib/stores/startup';
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
{/if}
