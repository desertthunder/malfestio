import type { Heading } from "$lib/wikilink";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { For, Show } from "solid-js";

type OutlinePanelProps = { headings: Heading[]; onHeadingClick?: (id: string) => void };

/**
 * Table of contents panel showing document outline from markdown headings
 */
export const OutlinePanel: Component<OutlinePanelProps> = (props) => {
  const handleClick = (id: string, e: MouseEvent) => {
    e.preventDefault();
    props.onHeadingClick?.(id);
    const element = document.getElementById(id);
    if (element) {
      element.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  };

  return (
    <div class="space-y-2">
      <h3 class="text-sm font-semibold text-slate-400 uppercase tracking-wide">Outline</h3>
      <Show when={props.headings.length > 0} fallback={<p class="text-sm text-slate-500 italic">No headings found</p>}>
        <nav class="space-y-1">
          <For each={props.headings}>
            {(heading) => (
              <A
                href={`#${heading.id}`}
                onClick={(e) => handleClick(heading.id, e)}
                class="block text-sm text-slate-300 hover:text-blue-400 transition-colors truncate"
                style={{ "padding-left": `${(heading.level - 1) * 12}px` }}>
                {heading.text}
              </A>
            )}
          </For>
        </nav>
      </Show>
    </div>
  );
};

export default OutlinePanel;
