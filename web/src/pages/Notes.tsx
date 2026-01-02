import { NoteCard } from "$components/NoteCard";
import { Button } from "$components/ui/Button";
import { EmptyState } from "$components/ui/EmptyState";
import { api } from "$lib/api";
import type { Note } from "$lib/model";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { createMemo, createResource, createSignal, For, Show } from "solid-js";

const fetchNotes = async (): Promise<Note[]> => {
  const res = await api.getNotes();
  if (!res.ok) return [];
  return res.json();
};

type ViewMode = "grid" | "list";

const Notes: Component = () => {
  const [notes] = createResource(fetchNotes);
  const [viewMode, setViewMode] = createSignal<ViewMode>("grid");
  const [searchQuery, setSearchQuery] = createSignal("");

  const filteredNotes = createMemo(() => {
    const allNotes = notes() || [];
    const query = searchQuery().toLowerCase().trim();
    if (!query) return allNotes;
    return allNotes.filter((note) =>
      note.title.toLowerCase().includes(query)
      || note.body.toLowerCase().includes(query)
      || note.tags.some((tag) => tag.toLowerCase().includes(query))
    );
  });

  return (
    <div class="max-w-7xl mx-auto p-6 space-y-6">
      <header class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">Notes</h1>
          <p class="text-slate-600 dark:text-slate-400 mt-1">Your personal knowledge base</p>
        </div>
        <A href="/notes/new">
          <Button variant="primary">New Note</Button>
        </A>
      </header>

      <div class="flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between">
        <div class="relative flex-1 max-w-md">
          <input
            type="text"
            placeholder="Search notes..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="w-full bg-slate-100 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg px-4 py-2 pl-10 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent" />
          <svg
            class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>

        <div class="flex items-center gap-2">
          <button
            onClick={() => setViewMode("grid")}
            class={`p-2 rounded ${
              viewMode() === "grid" ? "bg-slate-200 dark:bg-slate-700" : "hover:bg-slate-100 dark:hover:bg-slate-800"
            }`}
            aria-label="Grid view">
            <svg
              class="w-5 h-5 text-slate-600 dark:text-slate-300"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
            </svg>
          </button>
          <button
            onClick={() => setViewMode("list")}
            class={`p-2 rounded ${
              viewMode() === "list" ? "bg-slate-200 dark:bg-slate-700" : "hover:bg-slate-100 dark:hover:bg-slate-800"
            }`}
            aria-label="List view">
            <svg
              class="w-5 h-5 text-slate-600 dark:text-slate-300"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
        </div>
      </div>

      <Show
        when={!notes.loading}
        fallback={
          <div class="flex justify-center p-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
          </div>
        }>
        <Show
          when={filteredNotes().length > 0}
          fallback={
            <Show
              when={searchQuery()}
              fallback={
                <EmptyState
                  title="No notes yet"
                  description="Start capturing your thoughts and ideas"
                  action={
                    <A href="/notes/new">
                      <Button variant="primary">Create your first note</Button>
                    </A>
                  } />
              }>
              <EmptyState
                title="No matching notes"
                description={`No notes found for "${searchQuery()}"`}
                action={
                  <button
                    onClick={() => setSearchQuery("")}
                    class="text-sm font-medium text-blue-600 hover:text-blue-500 dark:text-blue-400">
                    Clear search
                  </button>
                } />
            </Show>
          }>
          <div
            class={viewMode() === "grid"
              ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
              : "flex flex-col gap-4"}>
            <For each={filteredNotes()}>{(note) => <NoteCard note={note} />}</For>
          </div>
        </Show>
      </Show>
    </div>
  );
};

export default Notes;
