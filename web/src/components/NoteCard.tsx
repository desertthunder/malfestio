import { useDensity } from "$lib/density-context";
import type { DensityMode } from "$lib/design-tokens";
import type { Note } from "$lib/model";
import { Card } from "$ui/Card";
import { Tag } from "$ui/Tag";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { For, Show } from "solid-js";

type NoteCardProps = { note: Note; density?: DensityMode };

export const NoteCard: Component<NoteCardProps> = (props) => {
  const globalDensity = useDensity();
  const density = () => props.density || globalDensity;

  const truncateBody = (body: string, maxLength: number) => {
    const plainText = body.replace(/[#*`[\]]/g, "").trim();
    return plainText.length > maxLength ? plainText.slice(0, maxLength) + "..." : plainText;
  };

  const paddingClass = () => {
    const d = density();
    return d === "compact" ? "p-4" : d === "spacious" ? "p-8" : "p-6";
  };

  return (
    <A href={`/notes/${props.note.id}`} class="block h-full no-underline group">
      <Card class="h-full flex flex-col hover:border-blue-400 dark:hover:border-blue-500 transition-colors">
        <div class={`${paddingClass()} flex-1 space-y-3`}>
          <div class="space-y-1">
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white line-clamp-1 group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
              {props.note.title || "Untitled"}
            </h3>
            <p class="text-xs text-slate-500 dark:text-slate-400">
              {new Date(props.note.updated_at).toLocaleDateString()}
            </p>
          </div>

          <p class="text-sm text-slate-600 dark:text-slate-300 line-clamp-3">{truncateBody(props.note.body, 120)}</p>

          <Show when={props.note.tags.length > 0}>
            <div class="flex flex-wrap gap-1.5 pt-2">
              <For each={props.note.tags.slice(0, 3)}>
                {(tag) => <Tag label={tag} color="blue" density={density()} />}
              </For>
              <Show when={props.note.tags.length > 3}>
                <span class="text-xs text-slate-400">+{props.note.tags.length - 3}</span>
              </Show>
            </div>
          </Show>
        </div>
      </Card>
    </A>
  );
};
