import type { CardType } from "$lib/store";
import { Button } from "$ui/Button";
import { createEffect, createSignal, Show } from "solid-js";

type CardEditorProps = {
  front?: string;
  back?: string;
  mediaUrl?: string;
  cardType?: CardType;
  hints?: string[];
  onSave: (data: { front: string; back: string; mediaUrl?: string; cardType: CardType; hints: string[] }) => void;
  onCancel?: () => void;
};

export function CardEditor(props: CardEditorProps) {
  const [front, setFront] = createSignal("");
  const [back, setBack] = createSignal("");
  const [mediaUrl, setMediaUrl] = createSignal("");
  const [cardType, setCardType] = createSignal<CardType>("basic");
  const [hints, setHints] = createSignal("");

  createEffect(() => {
    if (props.front) setFront(props.front);
    if (props.back) setBack(props.back);
    if (props.mediaUrl) setMediaUrl(props.mediaUrl);
    if (props.cardType) setCardType(props.cardType);
    if (props.hints) setHints(props.hints.join(", "));
  });

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    const hintsArray = hints().split(",").map(h => h.trim()).filter(h => h);
    props.onSave({
      front: front(),
      back: back(),
      mediaUrl: mediaUrl() || undefined,
      cardType: cardType(),
      hints: hintsArray,
    });
    if (!props.front) {
      setFront("");
      setBack("");
      setMediaUrl("");
      setCardType("basic");
      setHints("");
    }
  };

  return (
    <form onSubmit={handleSubmit} class="space-y-4 p-4 border border-gray-800 rounded bg-gray-900/50">
      <div class="flex gap-4 items-center">
        <label class="text-sm font-medium text-gray-400">Card Type</label>
        <div class="flex gap-4">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="cardType"
              value="basic"
              checked={cardType() === "basic"}
              onChange={() => setCardType("basic")}
              class="text-blue-500 focus:ring-blue-500" />
            <span class="text-gray-300">Basic</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="cardType"
              value="cloze"
              checked={cardType() === "cloze"}
              onChange={() => setCardType("cloze")}
              class="text-blue-500 focus:ring-blue-500" />
            <span class="text-gray-300">Cloze</span>
          </label>
        </div>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-400 mb-1">
          {cardType() === "cloze" ? "Text (use {{...}} for deletions)" : "Front"}
        </label>
        <textarea
          value={front()}
          onInput={(e) => setFront(e.target.value)}
          class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
          placeholder={cardType() === "cloze" ? "The capital of France is {{Paris}}." : "Front of card..."}
          rows={2}
          required />
      </div>

      <Show when={cardType() === "basic"}>
        <div>
          <label class="block text-sm font-medium text-gray-400 mb-1">Back</label>
          <textarea
            value={back()}
            onInput={(e) => setBack(e.target.value)}
            class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
            placeholder="Back of card..."
            rows={3}
            required />
        </div>
      </Show>

      <Show when={cardType() === "cloze"}>
        <div class="p-3 bg-gray-800/50 rounded border border-gray-700">
          <div class="text-xs text-gray-500 mb-1">Preview</div>
          <div class="text-gray-300">{front().replace(/\{\{([^}]+)\}\}/g, "[...]")}</div>
        </div>
      </Show>

      <div>
        <label class="block text-sm font-medium text-gray-400 mb-1">Hints (comma separated, optional)</label>
        <input
          type="text"
          value={hints()}
          onInput={(e) => setHints(e.target.value)}
          class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
          placeholder="First letter: P, Country in Europe..." />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-400 mb-1">Media URL (Optional)</label>
        <input
          type="url"
          value={mediaUrl()}
          onInput={(e) => setMediaUrl(e.target.value)}
          class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
          placeholder="https://..." />
      </div>

      <div class="flex justify-end gap-2">
        <Show when={props.onCancel}>
          <Button type="button" variant="ghost" onClick={props.onCancel}>Cancel</Button>
        </Show>
        <Button type="submit">{props.front ? "Update Card" : "Add Card"}</Button>
      </div>
    </form>
  );
}
