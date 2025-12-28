import { createSignal, For, Show } from "solid-js";
import { api } from "../lib/api";
import type { Visibility } from "../lib/store";
import { toast } from "../lib/toast";
import { CardEditor } from "./CardEditor";
import { Button } from "./ui/Button";

export function DeckEditor(props: { onSave?: (data: any) => void }) {
  const [title, setTitle] = createSignal("");
  const [description, setDescription] = createSignal("");
  const [visibilityType, setVisibilityType] = createSignal<string>("Private");
  const [sharedWith, setSharedWith] = createSignal("");

  const [cards, setCards] = createSignal<any[]>([]);
  const [showCardEditor, setShowCardEditor] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();

    let visibility: Visibility;
    if (visibilityType() === "SharedWith") {
      visibility = { type: "SharedWith", content: sharedWith().split(",").map(s => s.trim()).filter(s => s) };
    } else {
      visibility = { type: visibilityType() as "Private" | "Unlisted" | "Public" };
    }

    const payload = { title: title(), description: description(), tags: [], visibility, cards: cards() };

    if (props.onSave) {
      props.onSave(payload);
      return;
    }

    try {
      const _res = await api.post("/decks", payload);
      toast.success("Deck created!");
    } catch {
      toast.error("Failed to create deck");
    }
  };

  const addCard = (cardData: any) => {
    setCards([...cards(), cardData]);
    setShowCardEditor(false);
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
              id="description"
              value={description()}
              onInput={(e) => setDescription(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500" />
          </div>

          <div>
            <label for="visibility" class="block text-sm font-medium text-gray-400 mb-1">Visibility</label>
            <select
              id="visibility"
              value={visibilityType()}
              onChange={(e) => setVisibilityType(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
              aria-label="Visibility">
              <option value="Private">Private</option>
              <option value="Unlisted">Unlisted</option>
              <option value="Public">Public</option>
              <option value="SharedWith">Shared With...</option>
            </select>
          </div>

          <Show when={visibilityType() === "SharedWith"}>
            <div>
              <label class="block text-sm font-medium text-gray-400 mb-1">Share with DIDs (comma separated)</label>
              <input
                type="text"
                value={sharedWith()}
                onInput={(e) => setSharedWith(e.target.value)}
                class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
                placeholder="did:plc:..., did:plc:..." />
            </div>
          </Show>
        </div>

        <div class="pt-4 border-t border-gray-800">
          <h3 class="text-lg font-medium text-white mb-4">Cards ({cards().length})</h3>

          <div class="space-y-4 mb-4">
            <For each={cards()}>
              {(card, i) => (
                <div class="p-4 border border-gray-800 rounded bg-gray-900 flex justify-between items-center">
                  <div class="truncate pr-4 font-mono text-sm text-gray-300">{card.front}</div>
                  <div class="text-gray-500 text-xs">Card {i() + 1}</div>
                </div>
              )}
            </For>
          </div>

          <Show
            when={showCardEditor()}
            fallback={
              <Button type="button" variant="secondary" onClick={() => setShowCardEditor(true)} class="w-full">
                Add Card
              </Button>
            }>
            <CardEditor onSave={addCard} onCancel={() => setShowCardEditor(false)} />
          </Show>
        </div>

        <div class="pt-6 flex justify-end">
          <Button type="submit" size="lg">Create Deck</Button>
        </div>
      </form>
    </div>
  );
}
