import { NoteEditor } from "$components/NoteEditor";
import { api } from "$lib/api";
import { toast } from "$lib/toast";
import { Button } from "$ui/Button";
import { useNavigate } from "@solidjs/router";
import { createSignal, Show } from "solid-js";

export default function Import() {
  const navigate = useNavigate();
  const [url, setUrl] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [showEditor, setShowEditor] = createSignal(false);
  const [importedData, setImportedData] = createSignal<
    { title: string; markdown: string; metadata: { author?: string; publish_date?: string; source_url: string } } | null
  >(null);

  const handleImport = async (e: Event) => {
    e.preventDefault();
    setLoading(true);
    setImportedData(null);
    setShowEditor(false);
    try {
      const res = await api.post("/import/article", { url: url() });
      if (res.ok) {
        const data = await res.json();
        setImportedData({ title: data.title, markdown: data.markdown, metadata: data.metadata });
        toast.success("Article extracted!");
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

  const handleQuickSave = async () => {
    if (!importedData()) return;
    setSaving(true);
    try {
      const res = await api.saveImportedArticle({
        url: importedData()!.metadata.source_url,
        tags: ["imported", "article"],
        visibility: { type: "Private" },
      });
      if (res.ok) {
        const note = await res.json();
        toast.success("Article saved!");
        navigate(`/notes/${note.id}`);
      } else {
        const err = await res.json();
        toast.error(err.error || "Failed to save");
      }
    } catch (e) {
      console.error(e);
      toast.error("Network error");
    } finally {
      setSaving(false);
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

      <Show when={importedData() && !showEditor()}>
        <div class="pt-8 border-t border-gray-800 space-y-4">
          <div>
            <h2 class="text-2xl font-semibold text-white">{importedData()!.title}</h2>
            <Show when={importedData()!.metadata.author || importedData()!.metadata.publish_date}>
              <div class="flex gap-4 text-sm text-[#C6C6C6] mt-2">
                <Show when={importedData()!.metadata.author}>
                  <span>By {importedData()!.metadata.author}</span>
                </Show>
                <Show when={importedData()!.metadata.publish_date}>
                  <span>Published: {new Date(importedData()!.metadata.publish_date!).toLocaleDateString()}</span>
                </Show>
              </div>
            </Show>
          </div>

          <div class="bg-gray-800 rounded-lg p-4 max-h-96 overflow-y-auto">
            <p class="text-[#C6C6C6] text-sm whitespace-pre-wrap line-clamp-6">
              {importedData()!.markdown.slice(0, 500)}...
            </p>
          </div>

          <div class="flex gap-4">
            <Button onClick={handleQuickSave} disabled={saving()}>{saving() ? "Saving..." : "Quick Save"}</Button>
            <Button variant="secondary" onClick={() => setShowEditor(true)}>Edit First</Button>
          </div>
        </div>
      </Show>

      <Show when={showEditor() && importedData()}>
        <div class="pt-8 border-t border-gray-800">
          <h2 class="text-xl font-semibold text-white mb-4">Edit Before Saving</h2>
          <NoteEditor initialTitle={importedData()?.title} initialContent={importedData()?.markdown} />
        </div>
      </Show>
    </div>
  );
}
