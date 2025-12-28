import rehypeExternalLinks from "rehype-external-links";
import rehypeSanitize from "rehype-sanitize";
import rehypeStringify from "rehype-stringify";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import { createEffect, createSignal } from "solid-js";
import { unified } from "unified";
import { api } from "../lib/api";
import { toast } from "../lib/toast";
import { Button } from "./ui/Button";

interface NoteEditorProps {
  noteId?: string; // If editing existing
  initialTitle?: string;
  initialContent?: string;
}

export function NoteEditor(props: NoteEditorProps) {
  const [title, setTitle] = createSignal(props.initialTitle || "");
  const [content, setContent] = createSignal(props.initialContent || "");
  const [preview, setPreview] = createSignal("");
  const [tags, setTags] = createSignal(""); // Comma sep
  const [visibility, setVisibility] = createSignal("Private");

  const processor = unified().use(remarkParse) // .use(remarkWikiLink) // Would need a plugin for wikilinks -> links
    .use(remarkRehype).use(rehypeSanitize) // Safety first
    .use(rehypeExternalLinks, { target: "_blank", rel: ["nofollow"] }).use(rehypeStringify);

  createEffect(async () => {
    try {
      const file = await processor.process(content());
      setPreview(String(file));
    } catch (e) {
      console.error(e);
    }
  });

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    try {
      const payload = {
        title: title(),
        body: content(),
        tags: tags().split(",").map(t => t.trim()).filter(t => t),
        visibility: { type: visibility() as "Private" | "Public" },
      };

      await api.post("/notes", payload);
      toast.success("Note saved!");
      if (!props.noteId) {
        setTitle("");
        setContent("");
        setTags("");
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
                value={visibility()}
                onChange={e => setVisibility(e.target.value)}
                class="w-full bg-gray-800 border-gray-700 text-white rounded p-2">
                <option value="Private">Private</option>
                <option value="Public">Public</option>
              </select>
            </div>
          </div>

          <Button type="submit">Save Note</Button>
        </form>
      </div>

      <div class="space-y-4">
        <h2 class="text-xl font-semibold text-gray-300">Preview</h2>
        <div
          class="prose prose-invert max-w-none bg-gray-900/50 p-6 rounded border border-gray-800 min-h-[500px]"
          innerHTML={preview()} />
      </div>
    </div>
  );
}
