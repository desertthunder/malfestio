import { NoteEditor } from "$components/NoteEditor";
import { api } from "$lib/api";
import type { Note } from "$lib/model";
import { A, useParams } from "@solidjs/router";
import { createResource, Show } from "solid-js";

const NoteEdit = () => {
  const params = useParams<{ id: string }>();

  const [note] = createResource(() => params.id, async (id: string): Promise<Note | null> => {
    const res = await api.getNote(id);
    if (!res.ok) return null;
    return res.json();
  });

  return (
    <div class="max-w-6xl mx-auto">
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
              <h2 class="text-xl font-semibold text-white">Note not found</h2>
              <p class="text-slate-400 mt-2">This note may have been deleted.</p>
              <A href="/notes" class="text-blue-500 hover:text-blue-400 mt-4 inline-block">← Back to Notes</A>
            </div>
          }>
          {(n) => <NoteEditor noteId={n().id} initialTitle={n().title} initialContent={n().body} />}
        </Show>
      </Show>
    </div>
  );
};

export default NoteEdit;
