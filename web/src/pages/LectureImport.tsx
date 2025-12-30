import { NoteEditor } from "$components/NoteEditor";
import { Button } from "$ui/Button";
import { createSignal, Show } from "solid-js";

export default function LectureImport() {
  const [url, setUrl] = createSignal("");
  const [title, setTitle] = createSignal("");
  const [outline, setOutline] = createSignal("");
  const [timestamps, setTimestamps] = createSignal("");
  const [showEditor, setShowEditor] = createSignal(false);

  const handleCreate = (e: Event) => {
    e.preventDefault();
    setShowEditor(true);
  };

  const buildContent = () => {
    let content = "";
    if (url()) {
      content += `Source: [Lecture](${url()})\n\n`;
    }
    if (timestamps()) {
      content += "## Timestamps\n\n";
      timestamps().split("\n").filter(t => t.trim()).forEach(t => {
        content += `- ${t.trim()}\n`;
      });
      content += "\n";
    }
    if (outline()) {
      content += "## Outline\n\n";
      content += outline();
    }
    return content;
  };

  return (
    <div class="max-w-4xl mx-auto space-y-8">
      <div class="space-y-2">
        <h1 class="text-3xl font-light text-[#F4F4F4]">Import Lecture</h1>
        <p class="text-[#C6C6C6]">Create notes from lecture videos with outlines and timestamps.</p>
      </div>

      <Show when={!showEditor()}>
        <form onSubmit={handleCreate} class="space-y-6 p-6 border border-gray-800 rounded bg-gray-900/40">
          <div>
            <label class="block text-sm font-medium text-gray-400 mb-1">Lecture URL</label>
            <input
              type="url"
              value={url()}
              onInput={(e) => setUrl(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="https://youtube.com/watch?v=... or lecture platform URL" />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-400 mb-1">Title</label>
            <input
              type="text"
              value={title()}
              onInput={(e) => setTitle(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500"
              placeholder="Lecture title"
              required />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-400 mb-1">Timestamps (one per line)</label>
            <textarea
              value={timestamps()}
              onInput={(e) => setTimestamps(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
              placeholder="0:00 Introduction&#10;5:30 Main Topic&#10;15:00 Examples&#10;30:00 Summary"
              rows={5} />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-400 mb-1">Outline (Markdown)</label>
            <textarea
              value={outline()}
              onInput={(e) => setOutline(e.target.value)}
              class="w-full bg-gray-800 border-gray-700 text-white rounded p-2 focus:ring-blue-500 focus:border-blue-500 font-mono text-sm"
              placeholder="# Key Concepts&#10;&#10;- Point 1&#10;- Point 2&#10;&#10;## Details&#10;&#10;Write your notes here..."
              rows={10} />
          </div>

          <Button type="submit">Create Note from Lecture</Button>
        </form>
      </Show>

      <Show when={showEditor()}>
        <div class="border-t border-gray-800 pt-8">
          <div class="flex justify-between items-center mb-4">
            <h2 class="text-xl font-semibold text-white">Edit Lecture Note</h2>
            <Button variant="ghost" onClick={() => setShowEditor(false)}>← Back</Button>
          </div>
          <NoteEditor initialTitle={title()} initialContent={buildContent()} />
        </div>
      </Show>
    </div>
  );
}
