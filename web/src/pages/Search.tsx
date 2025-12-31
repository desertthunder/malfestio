import { SearchInput } from "$components/SearchInput";
import { Card } from "$components/ui/Card";
import { api } from "$lib/api";
import { asCard, asDeck, asNote, type SearchResult } from "$lib/model";
import { A, useSearchParams } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Match, Show, Switch } from "solid-js";

const Search: Component = () => {
  const [searchParams] = useSearchParams();
  const query = () => {
    const q = searchParams.q;
    return Array.isArray(q) ? q[0] : q || "";
  };

  const [results] = createResource(query, async (q) => {
    if (!q) return [];
    const res = await api.search(q);
    if (res.ok) return await res.json() as SearchResult[];
    return [];
  });

  return (
    <div class="container mx-auto p-4 space-y-6">
      <div class="flex flex-col md:flex-row gap-4 items-center justify-between">
        <h1 class="text-2xl font-bold">Search Results</h1>
        <div class="w-full md:w-96">
          <SearchInput initialQuery={query()} />
        </div>
      </div>

      <Show when={results.loading}>
        <div class="flex justify-center p-8">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600" />
        </div>
      </Show>

      <Show when={!results.loading && results()?.length === 0}>
        <div class="text-center text-gray-500 p-8">
          <div class="i-bi-search text-4xl mb-2 mx-auto" />
          <p>No results found for "{query()}"</p>
        </div>
      </Show>

      <div class="grid grid-cols-1 gap-4">
        <For each={results()}>
          {(result) => (
            <Card class="p-4 hover:shadow-md transition-shadow">
              <div class="flex items-start gap-4">
                <div class="mt-1">
                  <Show when={result.item_type === "deck"}>
                    <div class="i-bi-collection text-xl text-blue-500" title="Deck" />
                  </Show>
                  <Show when={result.item_type === "card"}>
                    <div class="i-bi-card-text text-xl text-green-500" title="Card" />
                  </Show>
                  <Show when={result.item_type === "note"}>
                    <div class="i-bi-journal-text text-xl text-yellow-500" title="Note" />
                  </Show>
                </div>
                <div class="flex-1">
                  <Switch>
                    <Match when={asDeck(result)}>
                      {(item) => (
                        <>
                          <A href={`/decks/${item().data.id}`} class="text-lg font-semibold hover:underline">
                            {item().data.title}
                          </A>
                          <p class="text-gray-600 dark:text-gray-300 text-sm mt-1">{item().data.description}</p>
                        </>
                      )}
                    </Match>
                    <Match when={asCard(result)}>
                      {(item) => (
                        <>
                          <A
                            href={`/decks/${item().data.deck_id}`}
                            class="text-lg font-semibold hover:underline block mb-1">
                            Card in Deck
                          </A>
                          <p class="text-sm">
                            <span class="font-medium">Front:</span> {item().data.front}
                          </p>
                          <p class="text-sm text-gray-500">
                            <span class="font-medium">Back:</span> {item().data.back}
                          </p>
                        </>
                      )}
                    </Match>
                    <Match when={asNote(result)}>
                      {(item) => (
                        <>
                          <A href={`/notes/${item().data.id}`} class="text-lg font-semibold hover:underline">
                            {item().data.title}
                          </A>
                          <div class="prose prose-sm dark:prose-invert mt-2 max-h-24 overflow-hidden truncate">
                            Content match
                          </div>
                        </>
                      )}
                    </Match>
                  </Switch>
                  <div class="mt-2 flex items-center gap-3 text-xs text-gray-400">
                    <span>Result Type: {result.item_type}</span>
                    <span>•</span>
                    <span>Score: {result.rank.toFixed(2)}</span>
                    <Show when={result.source === "remote"}>
                      <span class="flex items-center gap-1 text-blue-500 bg-blue-50 dark:bg-blue-900/20 px-2 py-0.5 rounded-full">
                        <i class="i-bi-globe2 text-xs" />
                        <span>Remote</span>
                      </span>
                    </Show>
                  </div>
                </div>
              </div>
            </Card>
          )}
        </For>
      </div>
    </div>
  );
};

export default Search;
