import { fadeIn, scaleIn } from "$lib/animations";
import { api } from "$lib/api";
import type { Card, CardType, CreateDeckPayload, Visibility } from "$lib/model";
import { toast } from "$lib/toast";
import { useTutorialTarget } from "$lib/TutorialProvider";
import { Button } from "$ui/Button";
import { createSignal, For, Show } from "solid-js";
import { Motion } from "solid-motionone";
import { CardEditor } from "./CardEditor";

type CardData = Card & { hints: string[]; cardType: CardType };

export function DeckEditor(props: { onSave?: (deck: CreateDeckPayload) => void }) {
  const [title, setTitle] = createSignal("");
  const [description, setDescription] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [visibilityType, setVisibilityType] = createSignal<Visibility["type"]>("Private");
  const [sharedWith, setSharedWith] = createSignal("");

  const [cards, setCards] = createSignal<Card[]>([]);
  const [showCardEditor, setShowCardEditor] = createSignal(false);

  const registerTutorialTarget = (id: string) => {
    try {
      return useTutorialTarget(id);
    } catch {
      return () => {};
    }
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();

    let visibility: Visibility;
    if (visibilityType() === "SharedWith") {
      visibility = { type: "SharedWith", content: sharedWith().split(",").map(s => s.trim()).filter(s => s) };
    } else {
      visibility = { type: visibilityType() as Exclude<Visibility["type"], "SharedWith"> };
    }

    const tagsArray = tags().split(",").map(t => t.trim()).filter(t => t);
    const payload = { title: title(), description: description(), tags: tagsArray, visibility, cards: cards() };

    if (props.onSave) {
      props.onSave(payload);
      return;
    }

    try {
      const res = await api.post("/decks", payload);
      if (res.ok) {
        toast.success("Deck created!");
      } else {
        toast.error("Failed to create deck");
      }
    } catch {
      toast.error("Network error creating deck");
    }
  };

  const addCard = (cardData: CardData) => {
    const card: Card = {
      front: cardData.front,
      back: cardData.back,
      mediaUrl: cardData.mediaUrl,
      cardType: cardData.cardType,
      hints: cardData.hints,
    };
    setCards([...cards(), card]);
    setShowCardEditor(false);
  };

  const removeCard = (index: number) => setCards(cards().filter((_, i) => i !== index));

  const moveCard = (from: number, to: number) => {
    if (to < 0 || to >= cards().length) return;
    const newCards = [...cards()];
    const [moved] = newCards.splice(from, 1);
    newCards.splice(to, 0, moved);
    setCards(newCards);
  };

  return (
    <div class="space-y-8">
      <form
        onSubmit={handleSubmit}
        class="space-y-4 max-w-3xl mx-auto p-6 border border-gray-800 rounded bg-gray-900/40">
        <div class="grid grid-cols-1 gap-6">
          <div>
            <label for="title" class="block text-sm font-medium text-gray-400 mb-1">Title</label>
            <input
              ref={registerTutorialTarget("title")}
              id="title"
              type="text"
              value={title()}
              onInput={(e) => setTitle(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
              required />
          </div>

          <div>
            <label for="description" class="block text-sm font-medium text-gray-400 mb-1">Description</label>
            <textarea
              ref={registerTutorialTarget("description")}
              id="description"
              value={description()}
              onInput={(e) => setDescription(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>

          <div>
            <label for="tags" class="block text-sm font-medium text-gray-400 mb-1">Tags (comma separated)</label>
            <input
              ref={registerTutorialTarget("tags")}
              id="tags"
              type="text"
              value={tags()}
              onInput={(e) => setTags(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="language, vocabulary, spanish..." />
          </div>

          <div>
            <label for="visibility" class="block text-sm font-medium text-gray-400 mb-1">Visibility</label>
            <select
              ref={registerTutorialTarget("visibility")}
              id="visibility"
              value={visibilityType()}
              onChange={(e) => setVisibilityType(e.target.value as Visibility["type"])}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
              aria-label="Visibility">
              <option value="Private">Private</option>
              <option value="Unlisted">Unlisted</option>
              <option value="Public">Public</option>
              <option value="SharedWith">Shared With...</option>
            </select>
          </div>

          <Show when={visibilityType() === "SharedWith"}>
            <Motion.div {...fadeIn}>
              <label class="block text-sm font-medium text-gray-400 mb-1">Share with DIDs (comma separated)</label>
              <input
                type="text"
                value={sharedWith()}
                onInput={(e) => setSharedWith(e.target.value)}
                class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="did:plc:..., did:plc:..." />
            </Motion.div>
          </Show>
        </div>

        <div class="pt-4 border-t border-gray-800">
          <h3 class="text-lg font-medium text-white mb-4">Cards ({cards().length})</h3>

          <div class="space-y-2 mb-4">
            <For each={cards()}>
              {(card, i) => (
                <div class="p-4 border border-gray-800 rounded bg-gray-900 flex justify-between items-center group">
                  <div class="flex items-center gap-3 flex-1 min-w-0">
                    <div class="flex flex-col gap-1">
                      <button
                        type="button"
                        onClick={() => moveCard(i(), i() - 1)}
                        disabled={i() === 0}
                        class="text-gray-500 hover:text-gray-300 disabled:opacity-30 disabled:cursor-not-allowed p-1">
                        ▲
                      </button>
                      <button
                        type="button"
                        onClick={() => moveCard(i(), i() + 1)}
                        disabled={i() === cards().length - 1}
                        class="text-gray-500 hover:text-gray-300 disabled:opacity-30 disabled:cursor-not-allowed p-1">
                        ▼
                      </button>
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="truncate font-mono text-sm text-gray-300">{card.front}</div>
                      <div class="text-xs text-gray-500 flex gap-2 mt-1">
                        <span class="uppercase">{card.cardType || "basic"}</span>
                        <Show when={card.hints && card.hints.length > 0}>
                          <span>• {card.hints?.length} hint(s)</span>
                        </Show>
                      </div>
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-gray-500 text-xs">#{i() + 1}</span>
                    <button
                      type="button"
                      onClick={() => removeCard(i())}
                      class="text-red-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity p-1">
                      ✕
                    </button>
                  </div>
                </div>
              )}
            </For>
          </div>

          <Show
            when={showCardEditor()}
            fallback={
              <Button
                ref={registerTutorialTarget("add-card")}
                type="button"
                variant="secondary"
                onClick={() => setShowCardEditor(true)}
                class="w-full">
                Add Card
              </Button>
            }>
            <Motion.div {...scaleIn}>
              <CardEditor onSave={addCard} onCancel={() => setShowCardEditor(false)} />
            </Motion.div>
          </Show>
        </div>

        <div class="pt-6 flex justify-end">
          <Button type="submit" size="lg">Create Deck</Button>
        </div>
      </form>
    </div>
  );
}
