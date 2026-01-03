import FileDropZone from "$components/import/FileDropZone";
import { api } from "$lib/api";
import { toast } from "$lib/toast";
import { createSignal, For, Show } from "solid-js";

type Chunk = { heading: string; content: string };
type ImportResponse = { filename: string; content: string; chunks: Chunk[] };

export default function LectureImport() {
  const [file, setFile] = createSignal<File | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [result, setResult] = createSignal<ImportResponse | null>(null);
  const [saving, setSaving] = createSignal<string | null>(null);

  const handleFileSelect = async (selectedFile: File) => {
    setFile(selectedFile);
    setError(null);
    setLoading(true);

    const formData = new FormData();
    formData.append("file", selectedFile);

    try {
      const response = await fetch("/api/import/lecture", { method: "POST", body: formData });

      if (!response.ok) {
        throw new Error(`Upload failed: ${response.statusText}`);
      }

      const data: ImportResponse = await response.json();
      setResult(data);
    } catch (err) {
      console.error(err);
      setError(err instanceof Error ? err.message : "An unknown error occurred");
      setFile(null);
    } finally {
      setLoading(false);
    }
  };

  const handleError = (msg: string) => {
    setError(msg);
  };

  const handleReset = () => {
    setFile(null);
    setResult(null);
    setError(null);
  };

  const createNote = async (chunk: Chunk) => {
    try {
      const res = await api.createNote({
        title: chunk.heading || "Untitled Chunk",
        body: chunk.content,
        tags: ["lecture-import"],
        visibility: { type: "Private" },
      });
      if (!res.ok) throw new Error("Failed to create note");
      toast.success("Note created!");
      return true;
    } catch (e) {
      console.error(e);
      toast.error("Failed to save note");
      return false;
    }
  };

  const handleSaveNote = async (chunk: Chunk, index: number) => {
    setSaving(index.toString());
    await createNote(chunk);
    setSaving(null);
  };

  const handleSaveAllNotes = async () => {
    const data = result();
    if (!data) return;

    setSaving("all");
    let successCount = 0;
    for (const chunk of data.chunks) {
      const success = await createNote(chunk);
      if (success) successCount++;
    }
    setSaving(null);
    if (successCount > 0) {
      toast.success(`Saved ${successCount} notes`);
    }
  };

  const handleCreateFlashcards = async () => {
    const data = result();
    if (!data) return;

    setSaving("cards");
    try {
      const cards = data.chunks.map((chunk) => ({
        front: chunk.heading || "Untitled Section",
        back: chunk.content,
        mediaUrl: undefined,
      }));

      const res = await api.createDeck({
        title: `Flashcards: ${data.filename}`,
        description: `Imported from ${data.filename}`,
        visibility: { type: "Private" },
        cards,
        tags: ["lecture-import"],
      });

      if (res.ok) {
        toast.success("Deck created with flashcards!");
      } else {
        throw new Error("Failed to create deck");
      }
    } catch (e) {
      console.error(e);
      toast.error("Failed to create flashcards");
    } finally {
      setSaving(null);
    }
  };

  return (
    <div class="max-w-4xl mx-auto p-6 space-y-8">
      <div class="space-y-2">
        <h1 class="text-3xl font-bold tracking-tight text-white">Import Lecture Notes</h1>
        <p class="text-neutral-400">Upload a PDF or DOCX file to extract text and generate chunks for flashcards.</p>
      </div>

      <Show when={error()}>
        <div class="p-4 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400">{error()}</div>
      </Show>

      <Show
        when={result()}
        fallback={
          <div class="space-y-4">
            <Show when={loading()}>
              <div class="flex flex-col items-center py-12 space-y-4">
                <div class="w-8 h-8 border-4 border-accent-500 border-t-transparent rounded-full animate-spin" />
                <p class="text-neutral-400">Processing {file()?.name}...</p>
              </div>
            </Show>
            <Show when={!loading()}>
              <FileDropZone
                onFileSelect={handleFileSelect}
                onError={handleError}
                accept=".pdf,.docx,.txt"
                maxSize={10 * 1024 * 1024} />
            </Show>
          </div>
        }>
        <div class="space-y-6">
          <div class="flex flex-col md:flex-row md:items-center justify-between gap-4 p-4 rounded-xl bg-neutral-800/50 border border-neutral-700">
            <div>
              <h2 class="text-xl font-semibold text-white">Extracted Content</h2>
              <p class="text-sm text-neutral-400">from {file()?.name}</p>
            </div>
            <div class="flex flex-wrap gap-2">
              <button
                onClick={handleSaveAllNotes}
                disabled={!!saving()}
                class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-500 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2">
                <span class="i-ri-file-text-line" />
                {saving() === "all" ? "Saving..." : "Save All to Notes"}
              </button>
              <button
                onClick={handleCreateFlashcards}
                disabled={!!saving()}
                class="px-4 py-2 text-sm font-medium text-white bg-purple-600 hover:bg-purple-500 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2">
                <span class="i-ri-gallery-view-2" />
                {saving() === "cards" ? "Creating..." : "Create Flashcards"}
              </button>
              <button
                onClick={handleReset}
                disabled={!!saving()}
                class="px-4 py-2 text-sm font-medium text-neutral-300 hover:text-white bg-neutral-700 hover:bg-neutral-600 rounded-lg transition-colors">
                Import Another
              </button>
            </div>
          </div>

          <div class="grid gap-6">
            <For each={result()?.chunks}>
              {(chunk, index) => (
                <div class="relative group p-6 rounded-xl bg-neutral-800/30 border border-neutral-700/50 hover:border-neutral-600 transition-colors space-y-3">
                  <div class="flex items-center justify-between gap-4">
                    <div class="flex items-center gap-2 overflow-hidden">
                      <span class="shrink-0 px-2 py-1 text-xs font-medium uppercase tracking-wider text-accent-400 bg-accent-400/10 rounded">
                        Section
                      </span>
                      <h3 class="font-medium text-lg text-white truncate" title={chunk.heading}>{chunk.heading}</h3>
                    </div>
                    <button
                      onClick={() => handleSaveNote(chunk, index())}
                      disabled={!!saving()}
                      class="shrink-0 opacity-0 group-hover:opacity-100 focus:opacity-100 px-3 py-1.5 text-xs font-medium text-neutral-300 hover:text-white bg-neutral-700/50 hover:bg-neutral-600 rounded transition-all flex items-center gap-1.5"
                      title="Save this chunk as a note">
                      <span class="i-ri-save-line" />
                      {saving() === index().toString() ? "Saving..." : "Save Note"}
                    </button>
                  </div>
                  <div class="prose prose-invert max-w-none text-neutral-300 text-sm whitespace-pre-wrap">
                    {chunk.content}
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}
