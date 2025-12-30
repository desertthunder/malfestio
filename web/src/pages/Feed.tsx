import { FollowButton } from "$components/social/FollowButton";
import { Button } from "$components/ui/Button";
import { Card } from "$components/ui/Card";
import { Dialog } from "$components/ui/Dialog";
import { EmptyState } from "$components/ui/EmptyState";
import { Skeleton } from "$components/ui/Skeleton";
import { Tabs } from "$components/ui/Tabs";
import { Tag } from "$components/ui/Tag";
import { api } from "$lib/api";
import type { Deck } from "$lib/model";
import { toast } from "$lib/toast";
import { A, useNavigate } from "@solidjs/router";
import { createResource, createSignal, For, Index, Match, Show, Switch } from "solid-js";
import { Motion } from "solid-motionone";

export default function Feed() {
  const navigate = useNavigate();
  const [forkDialogDeck, setForkDialogDeck] = createSignal<Deck | null>(null);

  const [followsFeed] = createResource(async () => {
    const res = await api.getFeedFollows();
    return res.ok ? ((await res.json()) as Deck[]) : [];
  });

  const [valuableFeed] = createResource(async () => {
    const res = await api.getFeedTrending();
    return res.ok ? ((await res.json()) as Deck[]) : [];
  });

  const handleFork = async () => {
    const deck = forkDialogDeck();
    if (!deck) return;
    try {
      const res = await api.forkDeck(deck.id);
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
      setForkDialogDeck(null);
    }
  };

  const DeckItem = (props: { deck: Deck; index: number }) => (
    <Motion.div
      initial={{ opacity: 0, y: 15 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, delay: props.index * 0.05 }}>
      <Card class="mb-4">
        <div class="flex justify-between items-start">
          <div class="flex-1">
            <h3 class="text-xl font-medium text-[#F4F4F4] mb-1">{props.deck.title}</h3>
            <p class="text-sm text-[#8D8D8D] mb-2">
              By {props.deck.owner_did} •{" "}
              <Show when={props.deck.published_at} fallback="Draft">
                {(published_at) => new Date(published_at()).toLocaleDateString()}
              </Show>
            </p>
            <p class="text-[#C6C6C6] mb-3 font-light">{props.deck.description}</p>
            <div class="flex gap-2 mb-3 flex-wrap">
              <For each={props.deck.tags}>{(tag) => <Tag label={tag} color="blue" />}</For>
            </div>
          </div>
          <div class="ml-4">
            <FollowButton did={props.deck.owner_did} />
          </div>
        </div>
        <div class="flex gap-2 items-center mt-4 pt-4 border-t border-[#393939]">
          <A href={`/decks/${props.deck.id}`}>
            <Button variant="secondary" size="sm">View</Button>
          </A>
          <Button variant="ghost" size="sm" onClick={() => setForkDialogDeck(props.deck)}>Fork</Button>
        </div>
      </Card>
    </Motion.div>
  );

  const DeckSkeleton = () => (
    <Card class="mb-4">
      <div class="flex justify-between items-start">
        <div class="flex-1 space-y-3">
          <Skeleton width="60%" height="1.5rem" />
          <Skeleton width="40%" height="0.875rem" />
          <Skeleton width="100%" height="1rem" />
          <div class="flex gap-2">
            <Skeleton width="4rem" height="1.5rem" rounded="full" />
            <Skeleton width="3rem" height="1.5rem" rounded="full" />
          </div>
        </div>
        <Skeleton width="5rem" height="2rem" />
      </div>
    </Card>
  );

  return (
    <Motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
      class="max-w-3xl mx-auto px-4 py-8">
      <Motion.div
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4 }}
        class="mb-8">
        <h1 class="text-4xl text-[#F4F4F4] tracking-tight mb-2">Discovery</h1>
        <p class="text-[#C6C6C6] font-light">Explore content from people you follow and trending decks.</p>
      </Motion.div>

      <Tabs tabs={[{ id: "following", label: "Following" }, { id: "trending", label: "Trending" }]}>
        {(activeTab) => (
          <Switch>
            <Match when={activeTab() === "following"}>
              <div class="mt-6">
                <Show when={!followsFeed.loading} fallback={<Index each={Array(3)}>{() => <DeckSkeleton />}</Index>}>
                  <For
                    each={followsFeed()}
                    fallback={
                      <EmptyState
                        title="No updates from followed users"
                        description="Follow some creators to see their latest decks here."
                        icon={<span class="i-bi-people text-4xl text-[#525252]" />} />
                    }>
                    {(deck, i) => <DeckItem deck={deck} index={i()} />}
                  </For>
                </Show>
              </div>
            </Match>
            <Match when={activeTab() === "trending"}>
              <div class="mt-6">
                <Show when={!valuableFeed.loading} fallback={<Index each={Array(3)}>{() => <DeckSkeleton />}</Index>}>
                  <For
                    each={valuableFeed()}
                    fallback={
                      <EmptyState
                        title="No trending decks"
                        description="Check back later for popular community content."
                        icon={<span class="i-bi-fire text-4xl text-[#525252]" />} />
                    }>
                    {(deck, i) => <DeckItem deck={deck} index={i()} />}
                  </For>
                </Show>
              </div>
            </Match>
          </Switch>
        )}
      </Tabs>

      <Dialog
        open={!!forkDialogDeck()}
        onClose={() => setForkDialogDeck(null)}
        title="Fork Deck"
        actions={
          <>
            <Button variant="ghost" onClick={() => setForkDialogDeck(null)}>Cancel</Button>
            <Button variant="primary" onClick={handleFork}>Fork Deck</Button>
          </>
        }>
        <p class="text-[#C6C6C6]">Are you sure you want to fork "{forkDialogDeck()?.title}"?</p>
        <p class="text-sm text-[#8D8D8D] mt-2">This will create a copy of this deck in your library.</p>
      </Dialog>
    </Motion.div>
  );
}
