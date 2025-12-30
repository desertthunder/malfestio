import { Card } from "$components/ui/Card";
import { EmptyState } from "$components/ui/EmptyState";
import { Skeleton } from "$components/ui/Skeleton";
import { Tag } from "$components/ui/Tag";
import { api } from "$lib/api";
import type { Deck } from "$lib/model";
import { Button } from "$ui/Button";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Index, Show } from "solid-js";
import { Motion } from "solid-motionone";

const DeckCard: Component<{ deck: Deck; index: number }> = (props) => (
  <Motion.div
    initial={{ opacity: 0, y: 20 }}
    animate={{ opacity: 1, y: 0 }}
    transition={{ duration: 0.4, delay: props.index * 0.05 }}>
    <Card class="h-full flex flex-col hover:border-[#0F62FE] transition-colors group">
      <div class="flex justify-between items-start mb-2">
        <h3 class="text-lg font-normal text-[#F4F4F4] group-hover:text-[#0F62FE] transition-colors line-clamp-1">
          {props.deck.title}
        </h3>
        <Show when={props.deck.visibility.type !== "Public"}>
          <Tag label={props.deck.visibility.type} color="gray" class="text-[10px]" />
        </Show>
      </div>
      <p class="text-sm text-[#C6C6C6] mb-6 line-clamp-2 grow font-light">{props.deck.description}</p>

      <div class="flex items-center gap-2 mb-4 flex-wrap">
        <For each={props.deck.tags}>{(tag) => <Tag label={`#${tag}`} color="blue" />}</For>
      </div>

      <div class="flex justify-end pt-4 border-t border-[#393939] mt-auto">
        <A
          href={`/decks/${props.deck.id}`}
          class="text-sm font-medium text-[#0F62FE] hover:text-[#0353E9] flex items-center gap-1">
          View Deck <span class="group-hover:translate-x-1 transition-transform">→</span>
        </A>
      </div>
    </Card>
  </Motion.div>
);

const DeckCardSkeleton: Component = () => (
  <Card class="h-full flex flex-col">
    <div class="flex justify-between items-start mb-2">
      <Skeleton width="60%" height="1.5rem" />
    </div>
    <div class="space-y-2 mb-6 grow">
      <Skeleton width="100%" height="0.875rem" />
      <Skeleton width="80%" height="0.875rem" />
    </div>
    <div class="flex gap-2 mb-4">
      <Skeleton width="4rem" height="1.5rem" rounded="full" />
      <Skeleton width="3rem" height="1.5rem" rounded="full" />
    </div>
    <div class="pt-4 border-t border-[#393939] mt-auto">
      <Skeleton width="5rem" height="1rem" />
    </div>
  </Card>
);

const Home: Component = () => {
  const [decks] = createResource(async () => {
    const res = await api.getDecks();
    return res.ok ? ((await res.json()) as Deck[]) : [];
  });

  return (
    <Motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
      class="max-w-7xl mx-auto px-0 py-8">
      <Motion.div
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4 }}
        class="flex justify-between items-end mb-12 border-b border-[#393939] pb-4">
        <div>
          <h1 class="text-4xl text-[#F4F4F4] tracking-tight mb-2">Library</h1>
          <p class="text-[#C6C6C6] font-light">Manage your study decks and discover new content.</p>
        </div>
        <A href="/decks/new">
          <Button class="flex items-center gap-2">
            <span class="i-bi-plus-lg" /> Create Deck
          </Button>
        </A>
      </Motion.div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <Show when={!decks.loading} fallback={<Index each={Array(6)}>{() => <DeckCardSkeleton />}</Index>}>
          <For
            each={decks()}
            fallback={
              <div class="col-span-full">
                <EmptyState
                  title="No decks found"
                  description="Create your first deck to get started with spaced repetition learning."
                  icon={<span class="i-bi-collection text-4xl text-[#525252]" />}
                  action={
                    <A href="/decks/new">
                      <Button>Create Your First Deck</Button>
                    </A>
                  } />
              </div>
            }>
            {(deck, i) => <DeckCard deck={deck} index={i()} />}
          </For>
        </Show>
      </div>
    </Motion.div>
  );
};

export default Home;
