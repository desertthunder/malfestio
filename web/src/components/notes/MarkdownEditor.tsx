import type { EditorFont } from "$components/NoteEditor";
import { type BundledTheme, codeToHtml } from "shiki";
import { type Component, createEffect, createSignal, For, onCleanup, onMount, Show } from "solid-js";

export type MarkdownEditorProps = {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  showLineNumbers?: boolean;
  font?: EditorFont;
  theme?: BundledTheme;
  class?: string;
  /** Ref callback to expose insertAtCursor method */
  ref?: (api: MarkdownEditorAPI) => void;
};

export type MarkdownEditorAPI = {
  insertAtCursor: (before: string, after?: string) => void;
  focus: () => void;
  getTextarea: () => HTMLTextAreaElement | undefined;
  insertTab: () => void;
};

function escapeHtml(text: string): string {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(
    /'/g,
    "&#039;",
  );
}

function getFontFamily(font: EditorFont | undefined): string {
  switch (font) {
    case "neon":
      return "Monaspace Neon, monospace";
    case "argon":
      return "Monaspace Argon, monospace";
    case "krypton":
      return "Monaspace Krypton, monospace";
    case "radon":
      return "Monaspace Radon, monospace";
    case "xenon":
      return "Monaspace Xenon, monospace";
    case "google":
      return "Google Sans Code, monospace";
    default:
      return "JetBrains Mono, monospace";
  }
}

/**
 * A markdown editor component with live syntax highlighting.
 *
 * Uses a layered approach: hidden textarea for input + visible div with highlighted HTML overlay.
 * Handles IME composition events to prevent disruption during CJK input.
 */
export const MarkdownEditor: Component<MarkdownEditorProps> = (props) => {
  let textareaRef: HTMLTextAreaElement | undefined;
  let overlayRef: HTMLDivElement | undefined;
  let containerRef: HTMLDivElement | undefined;

  const [highlightedHtml, setHighlightedHtml] = createSignal("");
  const [isComposing, setIsComposing] = createSignal(false);
  const [currentLine, setCurrentLine] = createSignal(1);

  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  const updateHighlight = async (text: string) => {
    if (isComposing()) return;

    try {
      const textToHighlight = text || " ";
      const html = await codeToHtml(textToHighlight, { lang: "markdown", theme: props.theme ?? "vitesse-dark" });
      setHighlightedHtml(html);
    } catch (e) {
      console.error("Highlight error:", e);
      setHighlightedHtml(`<pre><code>${escapeHtml(text)}</code></pre>`);
    }
  };

  const debouncedHighlight = (text: string) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => updateHighlight(text), 50);
  };

  const syncScroll = () => {
    if (textareaRef && overlayRef) {
      overlayRef.scrollTop = textareaRef.scrollTop;
      overlayRef.scrollLeft = textareaRef.scrollLeft;
    }
  };

  const handleCompositionStart = () => {
    setIsComposing(true);
  };

  const handleCompositionEnd = () => {
    setIsComposing(false);
    debouncedHighlight(props.value);
  };

  const handleInput = (e: InputEvent) => {
    const target = e.target as HTMLTextAreaElement;
    props.onChange(target.value);
    updateCurrentLine(target);
    if (!isComposing()) {
      debouncedHighlight(target.value);
    }
  };

  const updateCurrentLine = (textarea: HTMLTextAreaElement) => {
    const text = textarea.value.substring(0, textarea.selectionStart);
    const line = text.split("\n").length;
    setCurrentLine(line);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Tab") {
      e.preventDefault();
      insertTab();
    }
  };

  const handleSelect = () => {
    if (textareaRef) {
      updateCurrentLine(textareaRef);
    }
  };

  const insertTab = () => {
    insertAtCursor("  "); // 2-space soft tab
  };

  const insertAtCursor = (before: string, after: string = "") => {
    if (!textareaRef) return;

    const start = textareaRef.selectionStart;
    const end = textareaRef.selectionEnd;
    const text = props.value;
    const selectedText = text.substring(start, end);
    const newText = text.substring(0, start) + before + selectedText + after + text.substring(end);

    props.onChange(newText);

    setTimeout(() => {
      if (textareaRef) {
        textareaRef.focus();
        const newCursorPos = start + before.length + selectedText.length;
        textareaRef.setSelectionRange(newCursorPos, newCursorPos);
      }
    }, 0);
  };

  onMount(() => {
    if (props.ref) {
      props.ref({ insertAtCursor, focus: () => textareaRef?.focus(), getTextarea: () => textareaRef, insertTab });
    }

    updateHighlight(props.value);
  });

  onCleanup(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
  });

  createEffect(() => {
    const value = props.value;
    if (!isComposing()) {
      debouncedHighlight(value);
    }
  });

  const lineNumbers = () => Array.from({ length: Math.max(props.value.split("\n").length, 1) }, (_, i) => i + 1);
  const fontFamily = () => getFontFamily(props.font);

  return (
    <div ref={containerRef} class={`relative overflow-hidden ${props.class ?? ""}`} style={{ "min-height": "400px" }}>
      <div class="flex h-full">
        <Show when={props.showLineNumbers !== false}>
          <div
            class="bg-slate-900 border-r border-slate-700 text-slate-600 text-right px-2 py-3 select-none text-sm leading-relaxed shrink-0"
            style={{ "font-family": fontFamily() }}
            aria-hidden="true">
            <For each={lineNumbers()}>
              {(num) => <div class={num === currentLine() ? "text-blue-400" : ""}>{num}</div>}
            </For>
          </div>
        </Show>

        <div class="relative flex-1 min-h-full">
          <div
            ref={overlayRef}
            class="absolute inset-0 p-3 text-sm leading-relaxed overflow-hidden pointer-events-none whitespace-pre-wrap break-word"
            style={{ "font-family": fontFamily() }}>
            {/* eslint-disable-next-line solid/no-innerhtml */}
            <div class="shiki-editor-highlight" innerHTML={highlightedHtml()} />
          </div>

          <textarea
            ref={textareaRef}
            value={props.value}
            onInput={handleInput}
            onScroll={syncScroll}
            onKeyDown={handleKeyDown}
            onSelect={handleSelect}
            onClick={handleSelect}
            onCompositionStart={handleCompositionStart}
            onCompositionEnd={handleCompositionEnd}
            placeholder={props.placeholder}
            aria-label="Markdown editor"
            aria-multiline="true"
            class="absolute inset-0 w-full h-full p-3 text-sm leading-relaxed resize-none focus:outline-none bg-transparent text-transparent caret-white selection:bg-blue-500/30"
            style={{ "font-family": fontFamily(), "white-space": "pre-wrap", "word-break": "break-word" }}
            spellcheck={false} />
        </div>
      </div>
    </div>
  );
};

export default MarkdownEditor;
