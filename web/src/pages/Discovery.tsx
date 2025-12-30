import { SearchInput } from "$components/SearchInput";
import { api } from "$lib/api";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Show } from "solid-js";

// TODO: type discovery response
const Discovery: Component = () => {
  const [data] = createResource(async () => {
    const res = await api.getDiscovery();
    if (res.ok) return await res.json();
    return { top_tags: [] };
  });

  return (
    <div class="container mx-auto p-4 space-y-8">
      <div class="text-center space-y-4">
        <h1 class="text-4xl font-extrabold bg-linear-to-r from-blue-600 to-purple-600 dark:from-blue-400 dark:to-purple-400 text-transparent bg-clip-text">
          Discover Malfestio
        </h1>
        <p class="text-xl text-gray-600 dark:text-gray-300">Explore community decks and popular topics</p>
        <div class="max-w-2xl mx-auto">
          <SearchInput />
        </div>
      </div>

      <div class="space-y-4">
        <h2 class="text-2xl font-bold flex items-center gap-2">
          <div class="i-bi-tags-fill text-purple-500" />
          Top Tags
        </h2>

        <Show
          when={!data.loading}
          fallback={
            <div class="flex gap-2">
              <div class="h-8 w-24 bg-gray-200 dark:bg-gray-700 rounded animate-pulse" />
            </div>
          }>
          <div class="flex flex-wrap gap-3">
            <For each={data()?.top_tags}>
              {(tag: [string, number]) => (
                <A
                  href={`/search?q=${encodeURIComponent(tag[0])}`}
                  class="px-4 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-full hover:shadow-md hover:border-blue-500 dark:hover:border-blue-500 transition-all flex items-center gap-2 group">
                  <span class="font-medium text-gray-700 dark:text-gray-200 group-hover:text-blue-600 dark:group-hover:text-blue-400">
                    #{tag[0]}
                  </span>
                  <span class="text-xs text-gray-400 bg-gray-100 dark:bg-gray-700 px-1.5 py-0.5 rounded-full">
                    {tag[1]}
                  </span>
                </A>
              )}
            </For>
            <Show when={data()?.top_tags.length === 0}>
              <p class="text-gray-500">No tags found yet. Create some decks!</p>
            </Show>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default Discovery;
