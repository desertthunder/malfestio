import type { Note } from "$lib/model";
import { extractWikilinkTitles } from "$lib/wikilink";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { createMemo, For, Show } from "solid-js";

type FilterType = "all" | "linked" | "orphaned";

type NotesSidebarProps = {
  notes: Note[];
  selectedTag?: string;
  selectedFilter?: FilterType;
  onTagSelect?: (tag: string | undefined) => void;
  onFilterSelect?: (filter: FilterType) => void;
};

type TagCount = { tag: string; count: number };

export const NotesSidebar: Component<NotesSidebarProps> = (props) => {
  const tagCounts = createMemo<TagCount[]>(() => {
    const counts = new Map<string, number>();
    props.notes.forEach((note) => {
      note.tags.forEach((tag) => {
        counts.set(tag, (counts.get(tag) ?? 0) + 1);
      });
    });
    return Array.from(counts.entries()).map(([tag, count]) => ({ tag, count })).sort((a, b) => b.count - a.count);
  });

  const recentNotes = createMemo(() => {
    return [...props.notes].sort((a, b) => {
      const dateA = a.updated_at ?? a.created_at ?? "";
      const dateB = b.updated_at ?? b.created_at ?? "";
      return dateB.localeCompare(dateA);
    }).slice(0, 5);
  });

  const linkedCount = createMemo(() => {
    return props.notes.filter((note) => extractWikilinkTitles(note.body).length > 0).length;
  });

  const orphanedCount = createMemo(() => {
    return props.notes.filter((note) => extractWikilinkTitles(note.body).length === 0).length;
  });

  const filterButtonClass = (filter: FilterType) => {
    const isActive = props.selectedFilter === filter;
    return `px-3 py-1.5 text-sm rounded-md transition-colors ${
      isActive ? "bg-blue-500/20 text-blue-400 font-medium" : "text-slate-400 hover:text-white hover:bg-slate-800"
    }`;
  };

  return (
    <aside class="w-64 shrink-0 space-y-6" data-testid="notes-sidebar">
      {/* Quick Filters */}
      <section>
        <h3 class="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-2">Filters</h3>
        <div class="flex flex-col gap-1">
          <button
            class={filterButtonClass("all")}
            onClick={() => props.onFilterSelect?.("all")}
            data-testid="filter-all">
            All ({props.notes.length})
          </button>
          <button
            class={filterButtonClass("linked")}
            onClick={() => props.onFilterSelect?.("linked")}
            data-testid="filter-linked">
            Linked ({linkedCount()})
          </button>
          <button
            class={filterButtonClass("orphaned")}
            onClick={() => props.onFilterSelect?.("orphaned")}
            data-testid="filter-orphaned">
            Orphaned ({orphanedCount()})
          </button>
        </div>
      </section>

      {/* Tags */}
      <Show when={tagCounts().length > 0}>
        <section>
          <h3 class="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-2">Tags</h3>
          <div class="space-y-1">
            <For each={tagCounts().slice(0, 10)}>
              {(item) => (
                <button
                  class={`flex items-center justify-between w-full px-2 py-1 text-sm rounded transition-colors ${
                    props.selectedTag === item.tag
                      ? "bg-blue-500/20 text-blue-400"
                      : "text-slate-400 hover:text-white hover:bg-slate-800"
                  }`}
                  onClick={() => props.onTagSelect?.(props.selectedTag === item.tag ? undefined : item.tag)}
                  data-testid={`tag-${item.tag}`}>
                  <span class="truncate">#{item.tag}</span>
                  <span class="text-xs text-slate-500">{item.count}</span>
                </button>
              )}
            </For>
            <Show when={tagCounts().length > 10}>
              <p class="text-xs text-slate-500 px-2">+{tagCounts().length - 10} more tags</p>
            </Show>
          </div>
        </section>
      </Show>

      {/* Recent Notes */}
      <Show when={recentNotes().length > 0}>
        <section>
          <h3 class="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-2">Recent</h3>
          <div class="space-y-1">
            <For each={recentNotes()}>
              {(note) => (
                <A
                  href={`/notes/${note.id}`}
                  class="block px-2 py-1 text-sm text-slate-400 hover:text-white hover:bg-slate-800 rounded truncate transition-colors"
                  data-testid={`recent-${note.id}`}>
                  {note.title || "Untitled"}
                </A>
              )}
            </For>
          </div>
        </section>
      </Show>
    </aside>
  );
};
