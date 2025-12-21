<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorView, basicSetup } from 'codemirror';
  import { EditorState } from '@codemirror/state';
  import { yaml } from '@codemirror/lang-yaml';
  import { keymap } from '@codemirror/view';
  import { indentWithTab } from '@codemirror/commands';
  import { resolvedTheme } from "../../stores/theme";

  interface Props {
    content: string;
    readonly?: boolean;
    onchange?: (value: string) => void;
  }

  let { content, readonly = true, onchange }: Props = $props();

  let editorContainer: HTMLDivElement;
  let editorView: EditorView | null = null;

 // Get CodeMirror theme based on current theme
	function getEditorTheme() {
	const isDark = $resolvedTheme === "dark";
	return EditorView.theme({
    '&': {
      backgroundColor: isDark ? "#0a0a0a" : "#ffffff",
	  color: isDark ? "#e4e4e7" : "#1f2937",
      height: '100%',
    },
    '.cm-content': {
      fontFamily: 'Hack, Menlo, Monaco, "Courier New", monospace',
      fontSize: '12px',
      caretColor: '#00d4aa',
    },
    '.cm-cursor': {
      borderLeftColor: '#00d4aa',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: '#00d4aa',
    },
    '.cm-gutters': {
      backgroundColor: isDark ? "#111111" : "#f9fafb",
	  color: isDark ? "#555555" : "#9ca3af",
      border: 'none',
	  borderRight: isDark ? "1px solid #222222" : "1px solid #e5e7eb",
    },
    '.cm-activeLineGutter': {
	  backgroundColor: isDark ? "#1a1a1a" : "#f3f4f6",
    },
    '.cm-activeLine': {
      backgroundColor: isDark ? "#1a1a40" : "#f3f4f6",
    },
    '.cm-selectionBackground': {
      backgroundColor: '#00d4aa30 !important',
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: '#00d4aa30 !important',
    },
    '.cm-matchingBracket': {
      backgroundColor: '#00d4aa40',
      outline: 'none',
    },
    // YAML syntax colors
    '.cm-string': { color: '#22c55e' },
    '.cm-number': { color: '#f59e0b' },
    '.cm-keyword': { color: '#00d4aa' },
    '.cm-comment': { color: isDark ? '#555555' : '#9ca3af' },
    '.cm-propertyName': { color: '#3b82f6' },
    '.cm-punctuation': { color: isDark ? '#888888' : '#6b7280' },
    '.cm-atom': { color: '#a855f7' },
    '.cm-meta': { color: '#ef4444' },
    // Scrollbar styling
    '.cm-scroller': {
      overflow: 'auto',
    },
    '.cm-scroller::-webkit-scrollbar': {
      width: '8px',
      height: '8px',
    },
    '.cm-scroller::-webkit-scrollbar-track': {
      background: isDark ? '#111111' : '#f5f5f5',
    },
    '.cm-scroller::-webkit-scrollbar-thumb': {
      background: isDark ? '#333333' : '#d0d0d0',
      borderRadius: '4px',
    },
    '.cm-scroller::-webkit-scrollbar-thumb:hover': {
      background: isDark ? '#444444' : '#b0b0b0',
    },
  }, { dark: isDark });

  onMount(() => {
    const extensions = [
      basicSetup,
      yaml(),
      getEditorTheme(),
      keymap.of([indentWithTab]),
      EditorView.lineWrapping,
    ];

    if (readonly) {
      extensions.push(EditorState.readOnly.of(true));
    } else if (onchange) {
      extensions.push(EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          onchange(update.state.doc.toString());
        }
      }));
    }

    editorView = new EditorView({
      state: EditorState.create({
        doc: content || '',
        extensions,
      }),
      parent: editorContainer,
    });
  });

  onDestroy(() => {
    editorView?.destroy();
  });

  // Update content when prop changes
  $effect(() => {
	const theme = $resolvedTheme;
	if (editorView) {
		// Reconfigure editor with new theme
		editorView.dispatch({
		effects: [
			// Remove old theme and add new one
			EditorView.reconfigure([
			basicSetup,
			yaml(),
			getEditorTheme(),
			keymap.of([indentWithTab]),
			EditorView.lineWrapping,
			...(readonly ? [EditorState.readOnly.of(true)] : []),
			...(onchange ? [EditorView.updateListener.of((update) => {
				if (update.docChanged) {
				onchange(update.state.doc.toString());
				}
			})] : []),
			])
		]
		});
	}
});

</script>

<div bind:this={editorContainer} class="h-full w-full overflow-hidden"></div>

<style>
  :global(.cm-editor) {
    height: 100%;
  }
  :global(.cm-scroller) {
    font-family: Hack, Menlo, Monaco, 'Courier New', monospace !important;
  }
</style>

