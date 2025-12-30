import { CommentSection } from "$components/social/CommentSection";
import { FollowButton } from "$components/social/FollowButton";
import { Button } from "$components/ui/Button";
import { Dialog } from "$components/ui/Dialog";
import { api } from "$lib/api";
import type { Card, Deck } from "$lib/model";
import { toast } from "$lib/toast";
import { A, useNavigate, useParams } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, createSignal, For, Show } from "solid-js";

const DeckView: Component = () => {
  const params = useParams();
  const navigate = useNavigate();
  const [showForkDialog, setShowForkDialog] = createSignal(false);
  const [deck] = createResource(() => params.id, async (id) => {
    const res = await api.getDeck(id);
    return res.ok ? (await res.json() as Deck) : null;
  });
  const [cards] = createResource(() => params.id, async (id) => {
    const res = await api.getDeckCards(id);
    return res.ok ? (await res.json() as Card[]) : [];
  });

  const handleFork = async () => {
    if (deck()) {
      try {
        const res = await api.forkDeck(deck()!.id);
        if (res.ok) {
          const newDeck = await res.json();
          toast.success("Deck forked successfully!");
          navigate(`/decks/${newDeck.id}`);
        } else {
          toast.error("Failed to fork deck.");
        }
      } catch (e) {
        console.error(e);
        toast.error("Error forking deck.");
      } finally {
        setShowForkDialog(false);
      }
    }
  };

  return (
    <div class="max-w-4xl mx-auto px-6 py-12">
      <Show when={!deck.loading} fallback={<div class="text-[#8D8D8D] font-light">Loading deck...</div>}>
        <Show
          when={deck()}
          fallback={
            <div class="p-8 border border-red-900/50 bg-red-900/10 text-red-400">
              Deck not found or you don't have access.
            </div>
          }>
          {deckValue => (
            <>
              <div class="mb-12">
                <div class="flex justify-between items-start mb-4">
                  <h1 class="text-4xl font-light text-[#F4F4F4] tracking-tight">{deckValue().title}</h1>
                  <Show when={deckValue().visibility.type !== "Public"}>
                    <span class="text-xs uppercase font-bold tracking-widest px-2 py-1 bg-[#393939] text-[#C6C6C6]">
                      {deckValue().visibility.type}
                    </span>
                  </Show>
                </div>

                <div class="flex items-center gap-4 mb-6">
                  <div class="text-[#C6C6C6] font-light">By {deckValue().owner_did}</div>
                  <FollowButton did={deckValue().owner_did || ""} />
                </div>

                <p class="text-[#C6C6C6] mb-6 font-light">{deckValue().description}</p>

                <Show when={deckValue().tags.length > 0}>
                  <div class="flex gap-2 mb-8">
                    <For each={deckValue().tags}>
                      {(tag) => (
                        <span class="text-xs text-[#8D8D8D] bg-[#161616] px-2 py-1 border border-[#393939]">
                          #{tag}
                        </span>
                      )}
                    </For>
                  </div>
                </Show>

                <div class="flex gap-4 border-t border-[#393939] pt-6">
                  <button class="bg-[#0F62FE] hover:bg-[#0353E9] text-white px-6 py-3 font-medium text-sm transition-colors">
                    Study Deck (Coming Soon)
                  </button>
                  <Button
                    onClick={() => setShowForkDialog(true)}
                    variant="secondary"
                    class="border border-[#393939] text-[#F4F4F4] hover:bg-[#262626] px-6 py-3 font-medium text-sm transition-colors">
                    Fork Deck
                  </Button>
                  <A
                    href="/"
                    class="px-6 py-3 border border-[#393939] text-[#F4F4F4] hover:bg-[#262626] font-medium text-sm transition-colors">
                    Back to Library
                  </A>
                </div>
              </div>
              <div>
                <h2 class="text-xl font-medium text-[#F4F4F4] mb-6 border-b border-[#393939] pb-4">
                  Cards <Show when={cards()}>{value => value().length}</Show>
                </h2>

                <Show when={cards.loading}>
                  <div class="text-[#8D8D8D] font-light text-sm">Loading cards...</div>
                </Show>

                <div class="grid gap-4">
                  <For
                    each={cards()}
                    fallback={
                      <div class="text-center py-12 border border-dashed border-[#393939] text-[#8D8D8D] font-light italic">
                        No cards in this deck.
                      </div>
                    }>
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
                            <div class="text-[#C6C6C6]">
                              {card.back || <span class="italic opacity-50">Empty</span>}
                            </div>
                          </div>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </div>
              <div class="mt-12 pt-8 border-t border-[#393939]">
                <CommentSection deckId={deckValue().id} />
              </div>
            </>
          )}
        </Show>
      </Show>

      <Dialog
        open={showForkDialog()}
        onClose={() => setShowForkDialog(false)}
        title="Fork Deck"
        actions={
          <>
            <Button variant="ghost" onClick={() => setShowForkDialog(false)}>Cancel</Button>
            <Button variant="primary" onClick={handleFork}>Fork Deck</Button>
          </>
        }>
        <p>Are you sure you want to fork "{deck()?.title}"?</p>
        <p class="text-sm text-gray-400 mt-2">
          This will create a copy of this deck in your library that you can study and edit.
        </p>
      </Dialog>
    </div>
  );
};

export default DeckView;
