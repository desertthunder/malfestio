/* eslint-disable solid/no-innerhtml */
import { api } from "$lib/api";
import { toast } from "$lib/toast";
import { Button } from "$ui/Button";
import rehypeExternalLinks from "rehype-external-links";
import rehypeSanitize from "rehype-sanitize";
import rehypeStringify from "rehype-stringify";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { createEffect, createSignal, Show } from "solid-js";
import { unified } from "unified";

type NoteEditorProps = { noteId?: string; initialTitle?: string; initialContent?: string };

export function NoteEditor(props: NoteEditorProps) {
  const [title, setTitle] = createSignal(props.initialTitle || "");
  const [content, setContent] = createSignal(props.initialContent || "");
  const [preview, setPreview] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [visibilityType, setVisibilityType] = createSignal<string>("Private");
  const [sharedWith, setSharedWith] = createSignal("");

  const processor = unified().use(remarkParse).use(remarkRehype).use(rehypeSanitize).use(rehypeExternalLinks, {
    target: "_blank",
    rel: ["nofollow"],
  }).use(rehypeStringify);

  createEffect(() => {
    processor.process(content()).then((file) => setPreview(String(file))).catch((e) => console.error(e));
  });

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    try {
      let visibility;
      if (visibilityType() === "SharedWith") {
        visibility = { type: "SharedWith", content: sharedWith().split(",").map(s => s.trim()).filter(s => s) };
      } else {
        visibility = { type: visibilityType() };
      }

      const payload = {
        title: title(),
        body: content(),
        tags: tags().split(",").map(t => t.trim()).filter(t => t),
        visibility,
      };

      const res = await api.post("/notes", payload);
      if (res.ok) {
        toast.success("Note saved!");
        if (!props.noteId) {
          setTitle("");
          setContent("");
          setTags("");
          setVisibilityType("Private");
          setSharedWith("");
        }
      } else {
        toast.error("Failed to save note");
      }
    } catch (e) {
      console.error(e);
      toast.error("Failed to save note");
    }
  };

  return (
    <div class="max-w-4xl mx-auto p-6 grid grid-cols-1 md:grid-cols-2 gap-6">
      <div class="space-y-4">
        <h1 class="text-2xl font-bold text-white">Note Editor</h1>

        <form onSubmit={handleSubmit} class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-1">Title</label>
            <input
              type="text"
              value={title()}
              onInput={e => setTitle(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2"
              placeholder="Note Title"
              required />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-400 mb-1">Content (Markdown + [[WikiLinks]])</label>
            <textarea
              value={content()}
              onInput={e => setContent(e.target.value)}
              class="w-full h-96 bg-gray-800 border-gray-700 text-white rounded p-2 font-mono text-sm leading-relaxed"
              placeholder="# Heading&#10;&#10;Write your thoughts... Link to other notes with [[Title]]" />
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-medium text-gray-400 mb-1">Tags</label>
              <input
                type="text"
                value={tags()}
                onInput={e => setTags(e.target.value)}
                class="w-full bg-gray-800 border-gray-700 text-white rounded p-2"
                placeholder="rust, learning, ..." />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-400 mb-1">Visibility</label>
              <select
                value={visibilityType()}
                onChange={e => setVisibilityType(e.target.value)}
                class="w-full bg-gray-800 border-gray-700 text-white rounded p-2">
                <option value="Private">Private</option>
                <option value="Unlisted">Unlisted</option>
                <option value="Public">Public</option>
                <option value="SharedWith">Shared With...</option>
              </select>
            </div>
          </div>

          <Show when={visibilityType() === "SharedWith"}>
            <div>
              <label class="block text-sm font-medium text-gray-400 mb-1">Share with DIDs (comma separated)</label>
              <input
                type="text"
                value={sharedWith()}
                onInput={(e) => setSharedWith(e.target.value)}
                class="w-full bg-gray-800 border-gray-700 text-white rounded p-2"
                placeholder="did:plc:..., did:plc:..." />
            </div>
          </Show>

          <Button type="submit">Save Note</Button>
        </form>
      </div>

      <div class="space-y-4">
        <h2 class="text-xl font-semibold text-gray-300">Preview</h2>
        {}
        <div
          class="prose prose-invert max-w-none bg-gray-900/50 p-6 rounded border border-gray-800 min-h-[500px]"
          innerHTML={preview()} />
      </div>
    </div>
  );
}
