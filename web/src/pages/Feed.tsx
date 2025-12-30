import { FollowButton } from "$components/social/FollowButton";
import { Button } from "$components/ui/Button";
import { Card } from "$components/ui/Card";
import { Dialog } from "$components/ui/Dialog";
import { Tabs } from "$components/ui/Tabs";
import { api } from "$lib/api";
import type { Deck } from "$lib/model";
import { toast } from "$lib/toast";
import { A, useNavigate } from "@solidjs/router";
import { createResource, createSignal, For, Match, Show, Switch } from "solid-js";

export default function Feed() {
  const navigate = useNavigate();
  const [forkDialogDeck, setForkDialogDeck] = createSignal<Deck | null>(null);

  const [followsFeed] = createResource(async () => {
    const res = await api.getFeedFollows();
    return res.ok ? (await res.json() as Deck[]) : [];
  });

  const [valuableFeed] = createResource(async () => {
    const res = await api.getFeedTrending();
    return res.ok ? (await res.json() as Deck[]) : [];
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

  const DeckItem = (props: { deck: Deck }) => (
    <Card class="mb-4">
      <div class="flex justify-between items-start">
        <div>
          <h3 class="text-xl font-bold mb-1">{props.deck.title}</h3>
          <p class="text-sm text-gray-400 mb-2">
            By {props.deck.owner_did} •{" "}
            <Show when={props.deck.published_at} fallback="Draft">
              {published_at => new Date(published_at()).toLocaleDateString()}
            </Show>
          </p>
          <p class="mb-3">{props.deck.description}</p>
          <div class="flex gap-2 mb-3">
            <For each={props.deck.tags}>
              {(tag) => <span class="bg-gray-800 px-2 py-1 rounded text-xs">{tag}</span>}
            </For>
          </div>
        </div>
        <div class="ml-4">
          <FollowButton did={props.deck.owner_did} />
        </div>
      </div>
      <div class="flex gap-2 items-center mt-2">
        <A href={`/decks/${props.deck.id}`} class="no-underline">
          <Button variant="secondary" size="sm">View</Button>
        </A>
        <Button variant="ghost" size="sm" onClick={() => setForkDialogDeck(props.deck)}>Fork</Button>
      </div>
    </Card>
  );

  return (
    <div class="container mx-auto p-4 max-w-3xl">
      <h1 class="text-3xl font-bold mb-6">Discovery</h1>
      <Tabs tabs={[{ id: "following", label: "Following" }, { id: "trending", label: "Trending" }]}>
        {(activeTab) => (
          <Switch>
            <Match when={activeTab() === "following"}>
              <div class="mt-4">
                <Show when={followsFeed()}>
                  {feed => (
                    <For
                      each={feed()}
                      fallback={<div class="text-gray-500 py-8 text-center">No updates from followed users.</div>}>
                      {(deck) => <DeckItem deck={deck} />}
                    </For>
                  )}
                </Show>
              </div>
            </Match>
            <Match when={activeTab() === "trending"}>
              <div class="mt-4">
                <Show when={valuableFeed()}>
                  {feed => (
                    <For each={feed()} fallback={<div class="text-gray-500 py-8 text-center">No trending decks.</div>}>
                      {(deck) => <DeckItem deck={deck} />}
                    </For>
                  )}
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
        <p>Are you sure you want to fork "{forkDialogDeck()?.title}"?</p>
        <p class="text-sm text-gray-400 mt-2">This will create a copy of this deck in your library.</p>
      </Dialog>
    </div>
  );
}
