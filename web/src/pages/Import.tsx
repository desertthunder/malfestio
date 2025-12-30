import { NoteEditor } from "$components/NoteEditor";
import { api } from "$lib/api";
import { toast } from "$lib/toast";
import { Button } from "$ui/Button";
import { createSignal, Show } from "solid-js";

export default function Import() {
  const [url, setUrl] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [importedData, setImportedData] = createSignal<{ title: string; content: string } | null>(null);

  const handleImport = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setImportedData(null);
    try {
      const res = await api.post("/import/article", { url: url() });
      if (res.ok) {
        const data = await res.json();
        const content = `Source: [${data.title}](${data.url})\n\n${data.text}`;
        setImportedData({ title: data.title, content });
        toast.success("Article imported!");
      } else {
        const err = await res.json();
        toast.error(err.error || "Failed to import");
      }
    } catch (e) {
      console.error(e);
      toast.error("Network error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="max-w-4xl mx-auto space-y-8">
      <div class="space-y-2">
        <h1 class="text-3xl font-light text-[#F4F4F4]">Import Article</h1>
        <p class="text-[#C6C6C6]">Extract content from web pages to create notes.</p>
      </div>

      <form onSubmit={handleImport} class="flex gap-4">
        <input
          type="url"
          value={url()}
          onInput={(e) => setUrl(e.target.value)}
          class="flex-1 bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
          placeholder="https://example.com/article"
          required />
        <Button type="submit" disabled={loading()}>{loading() ? "Importing..." : "Import"}</Button>
      </form>

      <Show when={importedData()}>
        <div class="pt-8 border-t border-gray-800">
          <h2 class="text-xl font-semibold text-white mb-4">Create Note from Import</h2>
          <NoteEditor initialTitle={importedData()?.title} initialContent={importedData()?.content} />
        </div>
      </Show>
    </div>
  );
}
