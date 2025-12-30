import { FollowButton } from "$components/social/FollowButton";
import { Button } from "$components/ui/Button";
import { Card } from "$components/ui/Card";
import { Tabs } from "$components/ui/Tabs";
import { api } from "$lib/api";
import type { Deck } from "$lib/model";
import { A } from "@solidjs/router";
import { createResource, For, Match, Show, Switch } from "solid-js";

export default function Feed() {
  const [followsFeed] = createResource(async () => {
    const res = await api.getFeedFollows();
    return res.ok ? (await res.json() as Deck[]) : [];
  });

  const [valuableFeed] = createResource(async () => {
    const res = await api.getFeedTrending();
    return res.ok ? (await res.json() as Deck[]) : [];
  });

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
        <Button
          variant="ghost"
          size="sm"
          onClick={() => {
            // TODO: use modal or toast
            if (confirm("Fork this deck?")) {
              api.forkDeck(props.deck.id).then(() => alert("Forked successfully!"));
            }
          }}>
          Fork
        </Button>
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
    </div>
  );
}
