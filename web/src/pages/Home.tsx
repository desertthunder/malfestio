import { api } from "$lib/api";
import type { Deck } from "$lib/model";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Show } from "solid-js";

const DeckCard: Component<{ deck: Deck }> = (props) => (
  <div class="bg-[#262626] border border-[#393939] p-4 hover:border-[#0F62FE] transition-colors group relative h-full flex flex-col">
    <div class="flex justify-between items-start mb-2">
      <h3 class="text-lg font-normal text-[#F4F4F4] group-hover:text-[#0F62FE] transition-colors line-clamp-1">
        {props.deck.title}
      </h3>
      <Show when={props.deck.visibility.type !== "Public"}>
        <span class="text-[10px] uppercase font-bold tracking-widest px-2 py-0.5 bg-[#393939] text-[#C6C6C6]">
          {props.deck.visibility.type}
        </span>
      </Show>
    </div>
    <p class="text-sm text-[#C6C6C6] mb-6 line-clamp-2 flex-grow font-light">{props.deck.description}</p>

    <div class="flex items-center gap-2 mb-4 flex-wrap">
      <For each={props.deck.tags}>
        {(tag) => <span class="text-xs text-[#8D8D8D] bg-[#161616] px-2 py-0.5 border border-[#393939]">#{tag}</span>}
      </For>
    </div>

    <div class="flex justify-end pt-4 border-t border-[#393939] mt-auto">
      <A
        href={`/decks/${props.deck.id}`}
        class="text-sm font-medium text-[#0F62FE] hover:text-[#0353E9] flex items-center gap-1">
        View Deck <span class="group-hover:translate-x-1 transition-transform">→</span>
      </A>
    </div>
  </div>
);

const Home: Component = () => {
  const [decks] = createResource(async () => {
    const res = await api.getDecks();
    return res.ok ? (await res.json() as Deck[]) : [];
  });

  return (
    <div class="max-w-7xl mx-auto px-0 py-8">
      <div class="flex justify-between items-end mb-12 border-b border-[#393939] pb-4">
        <div>
          <h1 class="text-4xl font-light text-[#F4F4F4] tracking-tight mb-2">Library</h1>
          <p class="text-[#C6C6C6] font-light">Manage your study decks and discover new content.</p>
        </div>
        <A
          href="/decks/new"
          class="bg-[#0F62FE] hover:bg-[#0353E9] text-white px-6 py-3 font-medium text-sm transition-colors flex items-center gap-2">
          <span>+</span> Create Deck
        </A>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <Show
          when={!decks.loading}
          fallback={
            <div class="col-span-full h-32 flex items-center justify-center text-[#8D8D8D] font-light">
              Loading library...
            </div>
          }>
          <For
            each={decks()}
            fallback={
              <div class="col-span-full py-16 text-center border border-dashed border-[#393939] bg-[#262626]/50">
                <h3 class="text-lg font-medium text-[#F4F4F4] mb-2">No decks found</h3>
                <p class="text-sm text-[#C6C6C6] max-w-sm mx-auto font-light">
                  Create your first deck to get started with spaced repetition learning.
                </p>
              </div>
            }>
            {(deck) => <DeckCard deck={deck} />}
          </For>
        </Show>
      </div>
    </div>
  );
};

export default Home;
