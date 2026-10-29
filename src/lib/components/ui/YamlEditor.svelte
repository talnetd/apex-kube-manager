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
  function getEditorTheme(): EditorView.Theme {
	const isDark = $resolvedTheme !== "light";
	return EditorView.theme({
    '&': {
      backgroundColor: 'var(--ui-app)',
	  color: 'var(--ui-primary)',
      height: '100%',
    },
    '.cm-content': {
      fontFamily: 'var(--font-mono)',
      fontSize: '12px',
      caretColor: 'var(--ui-accent)',
    },
    '.cm-cursor': {
      borderLeftColor: 'var(--ui-accent)',
    },
    '&.cm-focused .cm-cursor': {
      borderLeftColor: 'var(--ui-accent)',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--ui-app)',
	  color: 'var(--ui-faint)',
      border: 'none',
	  borderRight: '1px solid var(--ui-line)',
    },
    '.cm-activeLineGutter': {
	  backgroundColor: 'var(--ui-hover)',
    },
    '.cm-activeLine': {
      backgroundColor: 'var(--ui-hover)',
    },
    '.cm-selectionBackground': {
      backgroundColor: 'color-mix(in srgb, var(--ui-accent) 30%, transparent) !important',
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: 'color-mix(in srgb, var(--ui-accent) 30%, transparent) !important',
    },
    '.cm-matchingBracket': {
      backgroundColor: 'var(--ui-active)',
      outline: 'none',
    },
    // YAML syntax colors
    '.cm-string': { color: 'var(--ui-success)' },
    '.cm-number': { color: 'var(--ui-warning)' },
    '.cm-keyword': { color: 'var(--ui-accent-text)' },
    '.cm-comment': { color: 'var(--ui-faint)' },
    '.cm-propertyName': { color: 'var(--ui-accent-text)' },
    '.cm-punctuation': { color: 'var(--ui-muted)' },
    '.cm-atom': { color: 'var(--ui-primary)' },
    '.cm-meta': { color: 'var(--ui-danger)' },
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
  }

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

