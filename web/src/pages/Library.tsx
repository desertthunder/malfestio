import { Card } from "$components/ui/Card";
import { EmptyState } from "$components/ui/EmptyState";
import { api } from "$lib/api";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Show } from "solid-js";

type LibraryItemData = {
  title?: string;
  description?: string;
  tags?: string[];
  at_uri?: string;
  front?: string;
  back?: string;
  body?: string;
};

type LibraryItemKind = "deck" | "note" | "card";

type LibraryItemSource = "local" | "remote";

type LibraryItem = {
  item_type: LibraryItemKind;
  item_id: string;
  creator_did: string;
  data: LibraryItemData;
  rank: number;
  source: LibraryItemSource;
};

const fetchFederatedContent = async () => {
  const res = await api.search("", 50, 0, "remote");
  if (!res.ok) return [];
  return (await res.json()) as LibraryItem[];
};

const LibraryItemCard: Component<{ item: LibraryItem }> = (props) => {
  return (
    <Card class="h-full flex flex-col hover:border-blue-400 dark:hover:border-blue-500 transition-colors">
      <div class="p-6 flex-1 space-y-4">
        <div class="flex items-start justify-between">
          <div class="space-y-1">
            <h3 class="text-lg font-semibold text-slate-900 dark:text-white line-clamp-1">{props.item.data.title}</h3>
            <p class="text-sm text-slate-500 dark:text-slate-400 font-mono">
              by {props.item.creator_did.slice(0, 12)}...
            </p>
          </div>
          <span class="inline-flex items-center rounded-full bg-indigo-50 dark:bg-indigo-900/30 px-2 py-1 text-xs font-medium text-indigo-700 dark:text-indigo-300 ring-1 ring-inset ring-indigo-700/10">
            Remote
          </span>
        </div>

        <p class="text-sm text-slate-600 dark:text-slate-300 line-clamp-3">
          {props.item.data.description || "No description provided."}
        </p>

        <Show when={props.item.data.tags}>
          {tags => (
            <div class="pt-4 flex items-center justify-between border-t border-slate-100 dark:border-slate-700 mt-auto">
              <div class="text-xs text-slate-500">
                <For each={tags()}>{(tag) => `#${tag}`}</For>
              </div>
            </div>
          )}
        </Show>
      </div>
    </Card>
  );
};

const Library: Component = () => {
  const [items] = createResource(fetchFederatedContent);

  return (
    <div class="max-w-7xl mx-auto p-6 space-y-8">
      <header class="space-y-4">
        <h1 class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">Federated Library</h1>
        <p class="text-slate-600 dark:text-slate-400">
          Discover content from across the AT Protocol network. These decks are indexed from other users and PDS
          instances.
        </p>
      </header>

      <Show
        when={!items.loading}
        fallback={
          <div class="flex justify-center p-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
          </div>
        }>
        <Show
          when={items() && items()!.length > 0}
          fallback={
            <EmptyState
              title="No federated content found"
              description="We couldn't find any remote decks in the index yet. Try following some users!"
              action={
                <button
                  onClick={() => window.location.href = "/discovery"}
                  class="text-sm font-medium text-blue-600 hover:text-blue-500 dark:text-blue-400 dark:hover:text-blue-300">
                  Go to Discovery
                </button>
              } />
          }>
          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <For each={items()}>
              {(item) => (
                <Show when={item.item_type === "deck"}>
                  <Show when={item.data.at_uri} fallback={<LibraryItemCard item={item} />}>
                    {at_uri => (
                      <A
                        href={`/library/preview?uri=${encodeURIComponent(at_uri())}`}
                        class="block h-full no-underline">
                        <LibraryItemCard item={item} />
                      </A>
                    )}
                  </Show>
                </Show>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
};

export default Library;
