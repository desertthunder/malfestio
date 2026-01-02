/* eslint-disable solid/no-innerhtml */
import { Button } from "$components/ui/Button";
import { api } from "$lib/api";
import type { Note } from "$lib/model";
import { Tag } from "$ui/Tag";
import { A, useParams } from "@solidjs/router";
import rehypeExternalLinks from "rehype-external-links";
import rehypeSanitize from "rehype-sanitize";
import rehypeStringify from "rehype-stringify";
import remarkParse from "remark-parse";
import remarkRehype from "remark-rehype";
import type { Component } from "solid-js";
import { createEffect, createResource, createSignal, For, Show } from "solid-js";
import { unified } from "unified";

const NoteView: Component = () => {
  const params = useParams<{ id: string }>();
  const [note] = createResource(() => params.id, async (id: string): Promise<Note | null> => {
    const res = await api.getNote(id);
    if (!res.ok) return null;
    return res.json();
  });
  const [renderedContent, setRenderedContent] = createSignal("");

  const processor = unified().use(remarkParse).use(remarkRehype).use(rehypeSanitize).use(rehypeExternalLinks, {
    target: "_blank",
    rel: ["nofollow"],
  }).use(rehypeStringify);

  const updateRenderedContent = async (n: Note) => {
    const file = await processor.process(n.body);
    setRenderedContent(String(file));
  };

  createEffect(() => {
    const n = note();
    if (n?.body) {
      updateRenderedContent(n).catch(console.error);
    }
  });

  return (
    <div class="max-w-5xl mx-auto p-6">
      <Show
        when={!note.loading}
        fallback={
          <div class="flex justify-center p-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
          </div>
        }>
        <Show
          when={note()}
          fallback={
            <div class="text-center py-12">
              <h2 class="text-xl font-semibold text-slate-900 dark:text-white">Note not found</h2>
              <p class="text-slate-600 dark:text-slate-400 mt-2">
                This note may have been deleted or you don't have access to it.
              </p>
              <A href="/notes" class="text-blue-600 hover:text-blue-500 mt-4 inline-block">← Back to Notes</A>
            </div>
          }>
          {(n) => (
            <div class="space-y-6">
              <nav class="flex items-center gap-2 text-sm text-slate-500 dark:text-slate-400">
                <A href="/notes" class="hover:text-blue-600 dark:hover:text-blue-400">Notes</A>
                <span>›</span>
                <span class="text-slate-900 dark:text-white">{n().title || "Untitled"}</span>
              </nav>

              <header class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4">
                <div class="space-y-2">
                  <h1 class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">
                    {n().title || "Untitled"}
                  </h1>
                  <div class="flex items-center gap-3 text-sm text-slate-500 dark:text-slate-400">
                    <span>Updated {new Date(n().updated_at).toLocaleDateString()}</span>
                    <Show when={n().visibility.type !== "Private"}>
                      <span class="inline-flex items-center rounded-full bg-green-50 dark:bg-green-900/30 px-2 py-0.5 text-xs font-medium text-green-700 dark:text-green-300">
                        {n().visibility.type}
                      </span>
                    </Show>
                  </div>
                </div>
                <div class="flex gap-2">
                  <A href={`/notes/edit/${n().id}`}>
                    <Button variant="secondary">Edit</Button>
                  </A>
                </div>
              </header>

              <Show when={n().tags.length > 0}>
                <div class="flex flex-wrap gap-2">
                  <For each={n().tags}>{(tag) => <Tag label={tag} color="blue" />}</For>
                </div>
              </Show>

              <article class="prose prose-slate dark:prose-invert max-w-none bg-white dark:bg-slate-800/50 rounded-xl p-8 border border-slate-200 dark:border-slate-700">
                <div innerHTML={renderedContent()} />
              </article>
            </div>
          )}
        </Show>
      </Show>
    </div>
  );
};

export default NoteView;
