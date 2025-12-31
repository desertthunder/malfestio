import { FollowButton } from "$components/social/FollowButton";
import { Button } from "$components/ui/Button";
import { Card } from "$components/ui/Card";
import { EmptyState } from "$components/ui/EmptyState";
import { Tag } from "$components/ui/Tag";
import { api } from "$lib/api";
import type { Card as CardType, Deck } from "$lib/model";
import { toast } from "$lib/toast";
import { useNavigate, useSearchParams } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Show } from "solid-js";
import { Motion } from "solid-motionone";

type RemoteDeckResponse = { deck: Deck; cards: CardType[] };

const DeckPreview: Component = () => {
  const [searchParams] = useSearchParams();
  const navigate = useNavigate();
  const uri = () => searchParams.uri as string;

  const [data] = createResource(uri, async (u) => {
    if (!u) return null;
    const res = await api.getRemoteDeck(u);
    // TODO: Toast on error
    if (!res.ok) return null;
    return (await res.json()) as RemoteDeckResponse;
  });

  const handleFork = async () => {
    // TODO: Implement `forkRemoteDeck` or update `forkDeck` to handle AT-URIs.
    toast.error("Forking remote decks is not yet supported.");
  };

  return (
    <div class="max-w-4xl mx-auto px-6 py-12">
      <Show
        when={!data.loading}
        fallback={
          <div class="flex justify-center">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
          </div>
        }>
        <Show
          when={data()}
          fallback={
            <EmptyState
              title="Deck not found"
              description="Could not load the requested remote deck."
              action={
                <Button variant="secondary" onClick={() => navigate("/library")}>Back to Library</Button>
              } />
          }>
          {(deckData) => (
            <Motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4 }}>
              <div class="mb-12">
                <div class="flex justify-between items-start mb-4">
                  <h1 class="text-4xl text-slate-100 font-bold tracking-tight">{deckData().deck.title}</h1>
                  <Tag label="Remote" color="blue" />
                </div>

                <div class="flex items-center gap-4 mb-6">
                  <div class="text-slate-400 font-light">By {deckData().deck.owner_did}</div>
                  <FollowButton did={deckData().deck.owner_did || ""} />
                </div>

                <p class="text-slate-300 mb-6 font-light">{deckData().deck.description}</p>

                <div class="flex gap-2 mb-8 flex-wrap">
                  <For each={deckData().deck.tags}>{(tag) => <Tag label={`#${tag}`} color="gray" />}</For>
                </div>

                <div class="flex gap-4 border-t border-slate-700 pt-6">
                  <Button onClick={handleFork} variant="secondary">
                    <span class="mr-2">Fork to Library</span>
                  </Button>
                  <Button
                    variant="ghost"
                    onClick={() => navigate("/library")}>
                    Back to Library
                  </Button>
                </div>
              </div>

              <h2 class="text-xl font-medium text-slate-200 mb-6 border-b border-slate-700 pb-4">
                Cards <span class="text-slate-500">({deckData().cards.length})</span>
              </h2>

              <div class="grid gap-4">
                <For each={deckData().cards}>
                  {(card, i) => (
                    <Card class="hover:border-slate-600 transition-colors">
                      <div class="flex justify-between items-start mb-2 text-xs text-slate-500 font-mono">
                        <span class="opacity-50">CARD {i() + 1}</span>
                      </div>
                      <div class="grid md:grid-cols-2 gap-8">
                        <div>
                          <div class="text-[10px] uppercase tracking-widest text-slate-500 mb-1">Front</div>
                          <div class="text-slate-200">{card.front}</div>
                        </div>
                        <div class="md:border-l md:border-slate-700 md:pl-8">
                          <div class="text-[10px] uppercase tracking-widest text-slate-500 mb-1">Back</div>
                          <div class="text-slate-400">{card.back}</div>
                        </div>
                      </div>
                    </Card>
                  )}
                </For>
                <Show when={deckData().cards.length === 0}>
                  <div class="text-center py-12 text-slate-500">
                    No cards indexed for this deck because we strictly respect remote privacy settings or the deck is
                    empty.
                  </div>
                </Show>
              </div>
            </Motion.div>
          )}
        </Show>
      </Show>
    </div>
  );
};

export default DeckPreview;
