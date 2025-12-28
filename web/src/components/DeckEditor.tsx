import { createSignal, Show } from "solid-js";
import { api } from "../lib/api";
import type { Visibility } from "../lib/store";
import { toast } from "../lib/toast";

export function DeckEditor(props: { onSave?: (data: any) => void }) {
  const [title, setTitle] = createSignal("");
  const [description, setDescription] = createSignal("");
  const [visibilityType, setVisibilityType] = createSignal<string>("Private");
  const [sharedWith, setSharedWith] = createSignal("");

  const handleSubmit = async (e: Event) => {
    e.preventDefault();

    let visibility: Visibility;
    if (visibilityType() === "SharedWith") {
      visibility = { SharedWith: sharedWith().split(",").map(s => s.trim()).filter(s => s) };
    } else {
      visibility = visibilityType() as Visibility;
    }

    const payload = { title: title(), description: description(), tags: [], visibility };

    if (props.onSave) {
      props.onSave(payload);
      return;
    }

    try {
      await api.post("/decks", payload);
      toast.success("Deck created!");
    } catch {
      toast.error("Failed to create deck");
    }
  };

  return (
    <form onSubmit={handleSubmit} class="space-y-4 max-w-md mx-auto p-4 border rounded">
      <div>
        <label for="title" class="block text-sm font-medium">Title</label>
        <input
          id="title"
          type="text"
          value={title()}
          onInput={(e) => setTitle(e.target.value)}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm border p-2"
          required />
      </div>

      <div>
        <label for="description" class="block text-sm font-medium">Description</label>
        <textarea
          id="description"
          value={description()}
          onInput={(e) => setDescription(e.target.value)}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm border p-2" />
      </div>

      <div>
        <label for="visibility" class="block text-sm font-medium">Visibility</label>
        <select
          id="visibility"
          value={visibilityType()}
          onChange={(e) => setVisibilityType(e.target.value)}
          class="mt-1 block w-full rounded-md border-gray-300 shadow-sm border p-2"
          aria-label="Visibility">
          <option value="Private">Private</option>
          <option value="Unlisted">Unlisted</option>
          <option value="Public">Public</option>
          <option value="SharedWith">Shared With...</option>
        </select>
      </div>

      <Show when={visibilityType() === "SharedWith"}>
        <div>
          <label class="block text-sm font-medium">Share with DIDs (comma separated)</label>
          <input
            type="text"
            value={sharedWith()}
            onInput={(e) => setSharedWith(e.target.value)}
            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm border p-2"
            placeholder="did:plc:..., did:plc:..." />
        </div>
      </Show>

      <button
        type="submit"
        class="inline-flex justify-center rounded-md border border-transparent bg-indigo-600 py-2 px-4 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none">
        Create Deck
      </button>
    </form>
  );
}
