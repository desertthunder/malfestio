import type { Note } from "$lib/model";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { For, Show } from "solid-js";

type BacklinksPanelProps = { backlinks: Note[] };

/**
 * Panel showing notes that link TO the current note (incoming references)
 */
export const BacklinksPanel: Component<BacklinksPanelProps> = (props) => {
  return (
    <div class="space-y-2">
      <h3 class="text-sm font-semibold text-slate-400 uppercase tracking-wide">
        Backlinks
        <Show when={props.backlinks.length > 0}>
          <span class="ml-1 text-slate-500">({props.backlinks.length})</span>
        </Show>
      </h3>
      <Show when={props.backlinks.length > 0} fallback={<p class="text-sm text-slate-500 italic">No incoming links</p>}>
        <ul class="space-y-1">
          <For each={props.backlinks}>
            {(note) => (
              <li>
                <A
                  href={`/notes/${note.id}`}
                  class="text-sm text-slate-300 hover:text-blue-400 flex items-center gap-1 transition-colors">
                  <span class="i-ri-arrow-left-up-line text-emerald-500" />
                  {note.title || "Untitled"}
                </A>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </div>
  );
};

export default BacklinksPanel;
