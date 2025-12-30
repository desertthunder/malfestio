import { CommentSection } from "$components/social/CommentSection";
import { FollowButton } from "$components/social/FollowButton";
import { Button } from "$components/ui/Button";
import { Card } from "$components/ui/Card";
import { Dialog } from "$components/ui/Dialog";
import { EmptyState } from "$components/ui/EmptyState";
import { Skeleton } from "$components/ui/Skeleton";
import { Tag } from "$components/ui/Tag";
import { api } from "$lib/api";
import type { Card as CardType, Deck } from "$lib/model";
import { toast } from "$lib/toast";
import { A, useNavigate, useParams } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, createSignal, For, Index, Show } from "solid-js";
import { Motion } from "solid-motionone";

const CardSkeleton: Component = () => (
  <Card>
    <div class="flex justify-between items-start mb-2">
      <Skeleton width="4rem" height="0.75rem" />
    </div>
    <div class="grid md:grid-cols-2 gap-8">
      <div class="space-y-2">
        <Skeleton width="3rem" height="0.625rem" />
        <Skeleton width="100%" height="1rem" />
      </div>
      <div class="space-y-2 md:border-l md:border-[#393939] md:pl-8">
        <Skeleton width="3rem" height="0.625rem" />
        <Skeleton width="100%" height="1rem" />
      </div>
    </div>
  </Card>
);

const DeckView: Component = () => {
  const params = useParams();
  const navigate = useNavigate();
  const [showForkDialog, setShowForkDialog] = createSignal(false);
  const [deck] = createResource(() => params.id, async (id) => {
    const res = await api.getDeck(id);
    return res.ok ? ((await res.json()) as Deck) : null;
  });
  const [cards] = createResource(() => params.id, async (id) => {
    const res = await api.getDeckCards(id);
    return res.ok ? ((await res.json()) as CardType[]) : [];
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
    <Motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
      class="max-w-4xl mx-auto px-6 py-12">
      <Show
        when={!deck.loading}
        fallback={
          <div class="space-y-6">
            <Skeleton width="60%" height="2.5rem" />
            <Skeleton width="40%" height="1rem" />
            <Skeleton width="100%" height="1rem" />
            <div class="flex gap-2">
              <Skeleton width="4rem" height="1.5rem" rounded="full" />
              <Skeleton width="3rem" height="1.5rem" rounded="full" />
            </div>
          </div>
        }>
        <Show
          when={deck()}
          fallback={
            <EmptyState
              title="Deck not found"
              description="This deck doesn't exist or you don't have access to view it."
              icon={<span class="i-bi-exclamation-triangle text-4xl text-red-400" />}
              action={
                <A href="/">
                  <Button variant="secondary">Back to Library</Button>
                </A>
              } />
          }>
          {(deckValue) => (
            <>
              <Motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4 }}
                class="mb-12">
                <div class="flex justify-between items-start mb-4">
                  <h1 class="text-4xl text-[#F4F4F4] tracking-tight">{deckValue().title}</h1>
                  <Show when={deckValue().visibility.type !== "Public"}>
                    <Tag label={deckValue().visibility.type} color="gray" />
                  </Show>
                </div>

                <div class="flex items-center gap-4 mb-6">
                  <div class="text-[#C6C6C6] font-light">By {deckValue().owner_did}</div>
                  <FollowButton did={deckValue().owner_did || ""} />
                </div>

                <p class="text-[#C6C6C6] mb-6 font-light">{deckValue().description}</p>

                <Show when={deckValue().tags.length > 0}>
                  <div class="flex gap-2 mb-8 flex-wrap">
                    <For each={deckValue().tags}>{(tag) => <Tag label={`#${tag}`} color="blue" />}</For>
                  </div>
                </Show>

                <div class="flex gap-4 border-t border-[#393939] pt-6">
                  <Button disabled>
                    <span class="i-bi-play-fill" /> Study Deck
                  </Button>
                  <Button onClick={() => setShowForkDialog(true)} variant="secondary">
                    <span class="i-bi-box-arrow-up-right" /> Fork Deck
                  </Button>
                  <A href="/">
                    <Button variant="ghost">Back to Library</Button>
                  </A>
                </div>
              </Motion.div>

              <Motion.div
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4, delay: 0.1 }}>
                <h2 class="text-xl font-medium text-[#F4F4F4] mb-6 border-b border-[#393939] pb-4">
                  Cards <Show when={cards()}>{(value) => <span class="text-[#8D8D8D]">({value().length})</span>}</Show>
                </h2>

                <Show when={!cards.loading} fallback={<Index each={Array(3)}>{() => <CardSkeleton />}</Index>}>
                  <div class="grid gap-4">
                    <For
                      each={cards()}
                      fallback={
                        <EmptyState
                          title="No cards in this deck"
                          description="Add some cards to start studying."
                          icon={<span class="i-bi-card-text text-4xl text-[#525252]" />} />
                      }>
                      {(card, i) => (
                        <Motion.div
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          transition={{ duration: 0.3, delay: i() * 0.03 }}>
                          <Card class="hover:border-[#525252] transition-colors">
                            <div class="flex justify-between items-start mb-2 text-xs text-[#8D8D8D] font-mono">
                              <span class="opacity-50">CARD {i() + 1}</span>
                            </div>
                            <div class="grid md:grid-cols-2 gap-8">
                              <div>
                                <div class="text-[10px] uppercase tracking-widest text-[#525252] mb-1">Front</div>
                                <div class="text-[#E0E0E0]">{card.front}</div>
                              </div>
                              <div class="md:border-l md:border-[#393939] md:pl-8">
                                <div class="text-[10px] uppercase tracking-widest text-[#525252] mb-1">Back</div>
                                <div class="text-[#C6C6C6]">
                                  {card.back || <span class="italic opacity-50">Empty</span>}
                                </div>
                              </div>
                            </div>
                          </Card>
                        </Motion.div>
                      )}
                    </For>
                  </div>
                </Show>
              </Motion.div>

              <Motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.4, delay: 0.2 }}
                class="mt-12 pt-8 border-t border-[#393939]">
                <CommentSection deckId={deckValue().id} />
              </Motion.div>
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
        <p class="text-[#C6C6C6]">Are you sure you want to fork "{deck()?.title}"?</p>
        <p class="text-sm text-[#8D8D8D] mt-2">
          This will create a copy of this deck in your library that you can study and edit.
        </p>
      </Dialog>
    </Motion.div>
  );
};

export default DeckView;
