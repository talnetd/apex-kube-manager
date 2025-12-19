<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import '@xterm/xterm/css/xterm.css';

  interface Props {
    namespace: string;
    podName: string;
    container?: string;
    shell?: string;
    onClose?: () => void;
  }

  let { namespace, podName, container, shell: initialShell, onClose }: Props = $props();

  // Common shells available in containers
  const SHELLS = [
    { value: '/bin/sh', label: '/bin/sh' },
    { value: '/bin/bash', label: '/bin/bash' },
    { value: '/bin/ash', label: '/bin/ash (Alpine)' },
    { value: '/bin/zsh', label: '/bin/zsh' },
  ];

  let terminalContainer: HTMLDivElement;
  let terminal: Terminal | null = null;
  let fitAddon: FitAddon | null = null;
  let sessionId: string | null = null;
  let isConnected = $state(false);
  let connectionError = $state<string | null>(null);
  let selectedShell = $state(initialShell || '/bin/sh');

  // Event listeners
  let unlistenData: UnlistenFn | null = null;
  let unlistenExit: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;

  onMount(() => {
    terminal = new Terminal({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: 'Hack, Menlo, Monaco, "Courier New", monospace',
      scrollback: 10000,
      theme: {
        background: '#0a0a0a',
        foreground: '#ffffff',
        cursor: '#00d4aa',
        cursorAccent: '#0a0a0a',
        selectionBackground: 'rgba(0, 212, 170, 0.3)',
        black: '#000000',
        red: '#ef4444',
        green: '#22c55e',
        yellow: '#f59e0b',
        blue: '#3b82f6',
        magenta: '#a855f7',
        cyan: '#00d4aa',
        white: '#ffffff',
        brightBlack: '#555555',
        brightRed: '#f87171',
        brightGreen: '#4ade80',
        brightYellow: '#fbbf24',
        brightBlue: '#60a5fa',
        brightMagenta: '#c084fc',
        brightCyan: '#5eead4',
        brightWhite: '#ffffff',
      },
    });

    fitAddon = new FitAddon();
    const webLinksAddon = new WebLinksAddon();

    terminal.loadAddon(fitAddon);
    terminal.loadAddon(webLinksAddon);

    terminal.open(terminalContainer);
    fitAddon.fit();

    // Write initial message
    terminal.writeln(`\x1b[36mConnecting to pod: ${podName}\x1b[0m`);
    terminal.writeln(`\x1b[90mNamespace: ${namespace}${container ? `, Container: ${container}` : ''}\x1b[0m`);
    terminal.writeln(`\x1b[90mShell: ${selectedShell}\x1b[0m`);
    terminal.writeln('');

    // Handle window resize
    const handleResize = () => {
      if (fitAddon && terminal) {
        fitAddon.fit();
        // Send resize to PTY
        if (sessionId) {
          const dims = fitAddon.proposeDimensions();
          if (dims) {
            invoke('pty_resize', {
              sessionId,
              rows: dims.rows,
              cols: dims.cols,
            }).catch(console.error);
          }
        }
      }
    };
    window.addEventListener('resize', handleResize);

    // Handle terminal input - send directly to PTY
    terminal.onData((data: string) => {
      if (sessionId && isConnected) {
        invoke('pty_write', { sessionId, data }).catch(console.error);
      }
    });

    // Start the PTY session
    startPty();

    return () => {
      window.removeEventListener('resize', handleResize);
      cleanup();
    };
  });

  async function startPty() {
    if (!terminal || !fitAddon) return;

    try {
      // Get terminal dimensions
      const dims = fitAddon.proposeDimensions();

      // Spawn PTY session
      sessionId = await invoke<string>('pty_spawn', {
        namespace,
        podName,
        container: container || null,
        shell: selectedShell,
      });

      // Set up event listeners for this session
      unlistenData = await listen<string>(`pty-data-${sessionId}`, (event) => {
        terminal?.write(event.payload);
      });

      unlistenExit = await listen(`pty-exit-${sessionId}`, () => {
        isConnected = false;
        terminal?.writeln('');
        terminal?.writeln('\x1b[33mConnection closed\x1b[0m');
      });

      unlistenError = await listen<string>(`pty-error-${sessionId}`, (event) => {
        connectionError = event.payload;
        terminal?.writeln(`\x1b[31mError: ${event.payload}\x1b[0m`);
      });

      isConnected = true;
      terminal.writeln('\x1b[32mConnected!\x1b[0m');
      terminal.writeln('');

      // Resize PTY to match terminal
      if (dims) {
        await invoke('pty_resize', {
          sessionId,
          rows: dims.rows,
          cols: dims.cols,
        });
      }

      // Focus terminal
      terminal.focus();

    } catch (error) {
      connectionError = String(error);
      terminal.writeln(`\x1b[31mFailed to connect: ${error}\x1b[0m`);
      terminal.writeln('');
      // Check if error is about shell not found
      if (String(error).includes('no such file or directory') || String(error).includes('exec format error')) {
        terminal.writeln(`\x1b[33mThe shell "${selectedShell}" may not exist in this container.\x1b[0m`);
        terminal.writeln('\x1b[90mTry a different shell: /bin/bash, /bin/ash (Alpine), or /bin/sh\x1b[0m');
        terminal.writeln('\x1b[90mNote: Distroless/scratch containers may not have any shell.\x1b[0m');
      } else {
        terminal.writeln('\x1b[90mMake sure kubectl is installed and configured.\x1b[0m');
      }
    }
  }

  async function cleanupPty() {
    // Unsubscribe from events
    if (unlistenData) {
      unlistenData();
      unlistenData = null;
    }
    if (unlistenExit) {
      unlistenExit();
      unlistenExit = null;
    }
    if (unlistenError) {
      unlistenError();
      unlistenError = null;
    }

    // Close PTY session
    if (sessionId) {
      try {
        await invoke('pty_close', { sessionId });
      } catch (e) {
        // Session might already be closed
      }
      sessionId = null;
    }
  }

  async function cleanup() {
    await cleanupPty();
    // Dispose terminal
    terminal?.dispose();
  }

  async function reconnect() {
    connectionError = null;
    isConnected = false;
    await cleanupPty();
    terminal?.reset();
    terminal?.writeln(`\x1b[36mReconnecting to pod: ${podName}\x1b[0m`);
    terminal?.writeln(`\x1b[90mShell: ${selectedShell}\x1b[0m`);
    terminal?.writeln('');
    await startPty();
  }

  onDestroy(() => {
    cleanup();
  });
</script>

<div class="flex flex-col h-full bg-bg-primary">
  <!-- Terminal Header -->
  <div class="flex items-center justify-between px-4 py-2 bg-bg-secondary border-b border-border-subtle">
    <div class="flex items-center gap-3">
      <svg class="w-4 h-4 text-accent-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
      <div>
        <span class="text-sm text-text-primary font-medium">{podName}</span>
        <span class="text-xs text-text-muted ml-2">{namespace}</span>
        {#if container}
          <span class="text-xs text-text-muted ml-2">/ {container}</span>
        {/if}
      </div>
      <!-- Connection status -->
      <div class="flex items-center gap-1.5 ml-4">
        <div class="w-2 h-2 rounded-full {isConnected ? 'bg-accent-success' : 'bg-accent-error'}"></div>
        <span class="text-xs text-text-muted">{isConnected ? 'Connected' : 'Disconnected'}</span>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <!-- Shell selector -->
      <select
        bind:value={selectedShell}
        disabled={isConnected}
        class="px-2 py-1 text-xs bg-bg-tertiary text-text-primary border border-border-subtle rounded focus:outline-none focus:border-accent-primary disabled:opacity-50 disabled:cursor-not-allowed"
        title={isConnected ? 'Disconnect to change shell' : 'Select shell'}
      >
        {#each SHELLS as shell}
          <option value={shell.value}>{shell.label}</option>
        {/each}
      </select>
      {#if !isConnected}
        <button
          onclick={reconnect}
          class="px-3 py-1 text-xs bg-accent-primary text-white rounded hover:bg-accent-primary/90 transition-colors"
        >
          Reconnect
        </button>
      {/if}
      {#if onClose}
        <button
          onclick={onClose}
          class="p-1 rounded hover:bg-bg-tertiary text-text-muted hover:text-text-primary transition-colors"
          title="Close"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <!-- Terminal Content -->
  <div bind:this={terminalContainer} class="flex-1 p-2"></div>
</div>

<style>
  :global(.xterm) {
    height: 100%;
  }
  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
  :global(.xterm-viewport::-webkit-scrollbar) {
    width: 8px;
  }
  :global(.xterm-viewport::-webkit-scrollbar-track) {
    background: #111111;
  }
  :global(.xterm-viewport::-webkit-scrollbar-thumb) {
    background: #333333;
    border-radius: 4px;
  }
</style>
