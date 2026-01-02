import type { Note } from "$lib/model";
import type { WikiLink } from "$lib/wikilink";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { For, Show } from "solid-js";

type WikilinksPanelProps = { links: WikiLink[]; notes: Note[]; resolveNote: (title: string) => Note | null };

/**
 * Panel showing outgoing wikilinks from the current note
 *
 * Displays links with status (resolved/unresolved)
 */
export const WikilinksPanel: Component<WikilinksPanelProps> = (props) => {
  const uniqueTitles = () => {
    const seen = new Set<string>();
    return props.links.filter((link) => {
      const normalized = link.title.toLowerCase();
      if (seen.has(normalized)) return false;
      seen.add(normalized);
      return true;
    });
  };

  return (
    <div class="space-y-2">
      <h3 class="text-sm font-semibold text-slate-400 uppercase tracking-wide">Wikilinks</h3>
      <Show when={props.links.length > 0} fallback={<p class="text-sm text-slate-500 italic">No outgoing links</p>}>
        <ul class="space-y-1">
          <For each={uniqueTitles()}>
            {(link) => {
              const resolved = () => props.resolveNote(link.title);
              return (
                <li class="text-sm">
                  <Show
                    when={resolved()}
                    fallback={
                      <span class="text-slate-500 flex items-center gap-1">
                        <span class="i-ri-link-unlink text-amber-500" />
                        <span class="line-through">{link.title}</span>
                      </span>
                    }>
                    {(note) => (
                      <A
                        href={`/notes/${note().id}`}
                        class="text-blue-400 hover:text-blue-300 flex items-center gap-1 transition-colors">
                        <span class="i-ri-link" />
                        {link.alias || link.title}
                      </A>
                    )}
                  </Show>
                </li>
              );
            }}
          </For>
        </ul>
      </Show>
    </div>
  );
};

export default WikilinksPanel;
