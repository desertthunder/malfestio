import { createSignal, Show } from "solid-js";
import { Button } from "./ui/Button";

type CardEditorProps = {
  front?: string;
  back?: string;
  mediaUrl?: string;
  onSave: (data: { front: string; back: string; mediaUrl?: string }) => void;
  onCancel?: () => void;
};

export function CardEditor(props: CardEditorProps) {
  const [front, setFront] = createSignal(props.front || "");
  const [back, setBack] = createSignal(props.back || "");
  const [mediaUrl, setMediaUrl] = createSignal(props.mediaUrl || "");

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    props.onSave({ front: front(), back: back(), mediaUrl: mediaUrl() || undefined });
    if (!props.front) {
      setFront("");
      setBack("");
      setMediaUrl("");
    }
  };

  return (
    <form onSubmit={handleSubmit} class="space-y-4 p-4 border border-gray-800 rounded bg-gray-900/50">
      <div>
        <label class="block text-sm font-medium text-gray-400 mb-1">Front</label>
        <textarea
          value={front()}
          onInput={(e) => setFront(e.target.value)}
          class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
          placeholder="Front of card..."
          rows={2}
          required />
      </div>

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
