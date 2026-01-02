/* eslint-disable solid/no-innerhtml */
import { EditorToolbar } from "$components/notes/EditorToolbar";
import { MarkdownEditor, type MarkdownEditorAPI } from "$components/notes/MarkdownEditor";
import { api } from "$lib/api";
import type { Note } from "$lib/model";
import { toast } from "$lib/toast";
import { Button } from "$ui/Button";
import rehypeShiki from "@shikijs/rehype";
import { useNavigate } from "@solidjs/router";
import { Textcomplete } from "@textcomplete/core";
import { TextareaEditor } from "@textcomplete/textarea";
import rehypeExternalLinks from "rehype-external-links";
import rehypeStringify from "rehype-stringify";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { createEffect, createResource, createSignal, onCleanup, Show } from "solid-js";
import { unified } from "unified";

export type EditorFont = "neon" | "argon" | "krypton" | "radon" | "xenon" | "jetbrains" | "google";

type NoteEditorProps = { noteId?: string; initialTitle?: string; initialContent?: string };

type EditorTab = "write" | "preview";

const processor = unified().use(remarkParse).use(remarkRehype).use(rehypeShiki, { theme: "vitesse-dark" }).use(
  rehypeExternalLinks,
  { target: "_blank", rel: ["nofollow"] },
).use(rehypeStringify);

export function NoteEditor(props: NoteEditorProps) {
  const navigate = useNavigate();
  const [title, setTitle] = createSignal(props.initialTitle || "");
  const [content, setContent] = createSignal(props.initialContent || "");
  const [preview, setPreview] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [visibilityType, setVisibilityType] = createSignal<string>("Private");
  const [sharedWith, setSharedWith] = createSignal("");
  const [showLineNumbers, setShowLineNumbers] = createSignal(true);
  const [editorFont, setEditorFont] = createSignal<EditorFont>("jetbrains");
  const [activeTab, setActiveTab] = createSignal<EditorTab>("write");

  let editorApi: MarkdownEditorAPI | undefined;
  let textcomplete: Textcomplete | undefined;

  const [allNotes] = createResource(async (): Promise<Note[]> => {
    const res = await api.getNotes();
    if (!res.ok) return [];
    return res.json();
  });

  const updatePreviewContent = async () => {
    const file = await processor.process(content());
    setPreview(String(file));
  };

  createEffect(() => {
    updatePreviewContent().catch(e => console.error(`Preview error: ${e instanceof Error ? e.message : e}`));
  });

  onCleanup(() => {
    textcomplete?.destroy();
  });

  const initTextcomplete = (api: MarkdownEditorAPI) => {
    editorApi = api;
    const textarea = api.getTextarea();
    if (!textarea) return;

    const editor = new TextareaEditor(textarea);
    textcomplete = new Textcomplete(editor, [{
      match: /\[\[([^\]]*)/,
      search: (term: string, callback: (results: string[]) => void) => {
        const notes = allNotes() ?? [];
        const filtered = notes.filter((n) => n.title.toLowerCase().includes(term.toLowerCase())).slice(0, 10).map((n) =>
          n.title
        );
        callback(filtered);
      },
      replace: (title: string) => `[[${title}]]`,
      template: (title: string) => title,
    }]);
  };

  const insertAtCursor = (before: string, after: string = "") => editorApi?.insertAtCursor(before, after);

  const handleBold = () => insertAtCursor("**", "**");
  const handleItalic = () => insertAtCursor("*", "*");
  const handleLink = () => insertAtCursor("[", "](url)");
  const handleCode = () => insertAtCursor("`", "`");
  const handleCodeBlock = () => insertAtCursor("```\n", "\n```");
  const handleWikilink = () => insertAtCursor("[[", "]]");
  const handleList = () => insertAtCursor("- ");
  const handleHeading = (level: 1 | 2 | 3 | 4 | 5 | 6) => insertAtCursor("#".repeat(level) + " ");

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.metaKey || e.ctrlKey) {
      switch (e.key) {
        case "b":
          e.preventDefault();
          handleBold();
          break;
        case "i":
          e.preventDefault();
          handleItalic();
          break;
        case "k":
          e.preventDefault();
          handleLink();
          break;
      }
    }
  };

  const handleEditorKeyDown = (e: KeyboardEvent) => handleKeyDown(e);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    try {
      let visibility;
      if (visibilityType() === "SharedWith") {
        visibility = { type: "SharedWith", content: sharedWith().split(",").map((s) => s.trim()).filter((s) => s) };
      } else {
        visibility = { type: visibilityType() };
      }

      const payload = {
        title: title(),
        body: content(),
        tags: tags().split(",").map((t) => t.trim()).filter((t) => t),
        visibility,
      };

      const res = props.noteId ? await api.updateNote(props.noteId, payload) : await api.post("/notes", payload);

      if (res.ok) {
        toast.success("Note saved!");
        if (props.noteId) {
          navigate(`/notes/${props.noteId}`);
        } else {
          try {
            const newNote = await res.json();
            navigate(`/notes/${newNote.id}`);
          } catch {
            navigate("/notes");
          }
        }
      } else {
        const errorText = await res.text();
        console.error("Failed to save note:", res.status, errorText);
        toast.error("Failed to save note");
      }
    } catch (e) {
      console.error(e);
      toast.error("Failed to save note");
    }
  };

  return (
    <div class="max-w-5xl mx-auto p-6">
      <div class="flex items-center justify-between mb-6">
        <h1 class="text-2xl font-bold text-white">{props.noteId ? "Edit Note" : "New Note"}</h1>

        <div class="flex items-center gap-4">
          <label class="flex items-center gap-2 text-sm text-slate-400">
            <input
              type="checkbox"
              checked={showLineNumbers()}
              onChange={(e) => setShowLineNumbers(e.target.checked)}
              class="rounded bg-slate-700 border-slate-600" />
            Line numbers
          </label>
          <select
            value={editorFont()}
            onChange={(e) => setEditorFont(e.target.value as EditorFont)}
            class="bg-slate-800 border-slate-700 text-white text-sm rounded px-2 py-1">
            <option value="jetbrains">JetBrains Mono</option>
            <option value="neon">Monaspace Neon</option>
            <option value="argon">Monaspace Argon</option>
            <option value="krypton">Monaspace Krypton</option>
            <option value="radon">Monaspace Radon</option>
            <option value="xenon">Monaspace Xenon</option>
            <option value="google">Google Sans Code</option>
          </select>
        </div>
      </div>

      <form onSubmit={handleSubmit} class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-slate-400 mb-1">Title</label>
          <input
            type="text"
            value={title()}
            onInput={(e) => setTitle(e.target.value)}
            class="w-full bg-slate-800 border border-slate-700 text-white rounded-lg p-3 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            placeholder="Note Title"
            required />
        </div>

        <div>
          <div class="flex border-b border-slate-700 mb-0">
            <button
              type="button"
              onClick={() => setActiveTab("write")}
              class={`px-4 py-2 text-sm font-medium transition-colors ${
                activeTab() === "write"
                  ? "text-white border-b-2 border-blue-500 -mb-px"
                  : "text-slate-400 hover:text-white"
              }`}>
              <span class="i-ri-edit-line mr-1" />
              Write
            </button>
            <button
              type="button"
              onClick={() => setActiveTab("preview")}
              class={`px-4 py-2 text-sm font-medium transition-colors ${
                activeTab() === "preview"
                  ? "text-white border-b-2 border-blue-500 -mb-px"
                  : "text-slate-400 hover:text-white"
              }`}>
              <span class="i-ri-eye-line mr-1" />
              Preview
            </button>
          </div>

          <Show when={activeTab() === "write"}>
            <div
              class="border border-slate-700 border-t-0 rounded-b-lg overflow-hidden"
              onKeyDown={handleEditorKeyDown}>
              <EditorToolbar
                onBold={handleBold}
                onItalic={handleItalic}
                onHeading={handleHeading}
                onLink={handleLink}
                onCode={handleCode}
                onCodeBlock={handleCodeBlock}
                onWikilink={handleWikilink}
                onList={handleList} />
              <MarkdownEditor
                value={content()}
                onChange={setContent}
                showLineNumbers={showLineNumbers()}
                font={editorFont()}
                ref={initTextcomplete}
                placeholder="# Heading

Write your thoughts...

Link to other notes with [[Title]]"
                class="bg-slate-800 min-h-[400px]" />
            </div>
          </Show>

          <Show when={activeTab() === "preview"}>
            <div
              class="prose prose-invert max-w-none bg-slate-800/50 p-6 rounded-b-lg border border-slate-700 border-t-0 min-h-[460px] overflow-auto"
              innerHTML={preview()} />
          </Show>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-slate-400 mb-1">Tags</label>
            <input
              type="text"
              value={tags()}
              onInput={(e) => setTags(e.target.value)}
              class="w-full bg-slate-800 border border-slate-700 text-white rounded-lg p-2"
              placeholder="rust, learning, ..." />
          </div>
          <div>
            <label class="block text-sm font-medium text-slate-400 mb-1">Visibility</label>
            <select
              value={visibilityType()}
              onChange={(e) => setVisibilityType(e.target.value)}
              class="w-full bg-slate-800 border border-slate-700 text-white rounded-lg p-2">
              <option value="Private">Private</option>
              <option value="Unlisted">Unlisted</option>
              <option value="Public">Public</option>
              <option value="SharedWith">Shared With...</option>
            </select>
          </div>
        </div>

        <Show when={visibilityType() === "SharedWith"}>
          <div>
            <label class="block text-sm font-medium text-slate-400 mb-1">Share with DIDs (comma separated)</label>
            <input
              type="text"
              value={sharedWith()}
              onInput={(e) => setSharedWith(e.target.value)}
              class="w-full bg-slate-800 border border-slate-700 text-white rounded-lg p-2"
              placeholder="did:plc:..., did:plc:..." />
          </div>
        </Show>

        <div class="flex justify-end">
          <Button type="submit">Save Note</Button>
        </div>
      </form>
    </div>
  );
}
