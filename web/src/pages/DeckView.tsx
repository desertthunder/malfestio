import { A, useParams } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Show } from "solid-js";
import { api } from "../lib/api";
import type { Visibility } from "../lib/store";

type Deck = {
  id: string;
  title: string;
  description: string;
  tags: string[];
  visibility: Visibility;
  owner_did: string;
};

type Card = { id: string; front: string; back?: string };

const fetchDeck = async (id: string): Promise<Deck | null> => {
  const res = await api.get(`/decks/${id}`);
  if (!res.ok) return null;
  return res.json();
};

const fetchCards = async (id: string): Promise<Card[]> => {
  const res = await api.get(`/decks/${id}/cards`);
  if (!res.ok) return [];
  return res.json();
};

const DeckView: Component = () => {
  const params = useParams();

  const [deck] = createResource(() => params.id, fetchDeck);
  const [cards] = createResource(() => params.id, fetchCards);

  return (
    <div class="max-w-4xl mx-auto px-6 py-12">
      <Show when={deck.loading}>
        <div class="text-[#8D8D8D] font-light">Loading deck...</div>
      </Show>

      <Show when={!deck.loading && deck() === null}>
        <div class="p-8 border border-red-900/50 bg-red-900/10 text-red-400">
          Deck not found or you don't have access.
        </div>
      </Show>

      <Show when={deck()}>
        <div class="mb-12">
          <div class="flex justify-between items-start mb-4">
            <h1 class="text-4xl font-light text-[#F4F4F4] tracking-tight">{deck()?.title}</h1>
            <Show when={deck()?.visibility.type !== "Public"}>
              <span class="text-xs uppercase font-bold tracking-widest px-2 py-1 bg-[#393939] text-[#C6C6C6]">
                {deck()?.visibility.type}
              </span>
            </Show>
          </div>

          <p class="text-[#C6C6C6] mb-6 font-light">{deck()?.description}</p>

          <div class="flex gap-2 mb-8">
            <For each={deck()?.tags}>
              {(tag) => (
                <span class="text-xs text-[#8D8D8D] bg-[#161616] px-2 py-1 border border-[#393939]">#{tag}</span>
              )}
            </For>
          </div>

          <div class="flex gap-4 border-t border-[#393939] pt-6">
            {/* Placeholder for Study Action */}
            <button class="bg-[#0F62FE] hover:bg-[#0353E9] text-white px-6 py-3 font-medium text-sm transition-colors">
              Study Deck (Coming Soon)
            </button>
            <A
              href="/"
              class="px-6 py-3 border border-[#393939] text-[#F4F4F4] hover:bg-[#262626] font-medium text-sm transition-colors">
              Back to Library
            </A>
          </div>
        </div>

        <div>
          <h2 class="text-xl font-medium text-[#F4F4F4] mb-6 border-b border-[#393939] pb-4">
            Cards ({cards()?.length || 0})
          </h2>

          <Show when={cards.loading}>
            <div class="text-[#8D8D8D] font-light text-sm">Loading cards...</div>
          </Show>

          <div class="grid gap-4">
            <For each={cards()}>
              {(card, i) => (
                <div class="p-6 bg-[#262626] border border-[#393939] hover:border-[#525252] transition-colors group">
                  <div class="flex justify-between items-start mb-2 text-xs text-[#8D8D8D] font-mono">
                    <span class="opacity-50">CARD {i() + 1}</span>
                  </div>
                  <div class="grid md:grid-cols-2 gap-8">
                    <div class="prose prose-invert prose-sm max-w-none">
                      <div class="text-[10px] uppercase tracking-widest text-[#525252] mb-1">Front</div>
                      <div class="text-[#E0E0E0]">{card.front}</div>
                    </div>
                    <div class="prose prose-invert prose-sm max-w-none md:border-l md:border-[#393939] md:pl-8">
                      <div class="text-[10px] uppercase tracking-widest text-[#525252] mb-1">Back</div>
                      <div class="text-[#C6C6C6]">{card.back || <span class="italic opacity-50">Empty</span>}</div>
                    </div>
                  </div>
                </div>
              )}
            </For>

            <Show when={!cards.loading && cards()?.length === 0}>
              <div class="text-center py-12 border border-dashed border-[#393939] text-[#8D8D8D] font-light italic">
                No cards in this deck.
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default DeckView;
