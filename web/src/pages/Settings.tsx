import { api } from "$lib/api";
import type { Component } from "solid-js";
import { createSignal, Show } from "solid-js";

const Settings: Component = () => {
  const [exportingDecks, setExportingDecks] = createSignal(false);
  const [exportingNotes, setExportingNotes] = createSignal(false);
  const [exportError, setExportError] = createSignal<string | null>(null);

  const handleExport = async (collection: "decks" | "notes") => {
    if (collection === "decks") setExportingDecks(true);
    else setExportingNotes(true);
    setExportError(null);

    try {
      const res = await api.exportData(collection);
      if (!res.ok) throw new Error(`Failed to export ${collection}`);

      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `malfestio_${collection}_export.json`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      a.remove();
    } catch (e) {
      console.error(e);
      setExportError(`Failed to export ${collection}. Please try again.`);
    } finally {
      if (collection === "decks") setExportingDecks(false);
      else setExportingNotes(false);
    }
  };

  return (
    <div class="max-w-4xl mx-auto p-6 space-y-8">
      <header class="space-y-4">
        <h1 class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">Settings</h1>
        <p class="text-slate-600 dark:text-slate-400">Manage your account preferences and data.</p>
      </header>

      <div class="grid gap-6">
        {/* Export Section */}
        <section class="p-6 bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700">
          <h2 class="text-xl font-semibold text-slate-900 dark:text-white mb-4">Export Data</h2>
          <p class="text-sm text-slate-600 dark:text-slate-400 mb-6">
            Download your data as JSON files. This includes all your content but excludes media files which are linked
            remotely.
          </p>

          <Show when={exportError()}>
            <div class="mb-4 p-4 text-sm text-red-600 bg-red-50 dark:bg-red-900/20 dark:text-red-400 rounded-lg border border-red-200 dark:border-red-900/50">
              {exportError()}
            </div>
          </Show>

          <div class="flex gap-4">
            <button
              onClick={() => handleExport("decks")}
              disabled={exportingDecks()}
              class="inline-flex items-center justify-center rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors">
              {exportingDecks() ? "Exporting..." : "Export Decks"}
            </button>
            <button
              onClick={() => handleExport("notes")}
              disabled={exportingNotes()}
              class="inline-flex items-center justify-center rounded-lg bg-white dark:bg-slate-700 px-4 py-2 text-sm font-medium text-slate-700 dark:text-slate-200 border border-slate-300 dark:border-slate-600 hover:bg-slate-50 dark:hover:bg-slate-600 focus:outline-none focus:ring-2 focus:ring-slate-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors">
              {exportingNotes() ? "Exporting..." : "Export Notes"}
            </button>
          </div>
        </section>

        {/* Preferences Section - Placeholder for now as per plan */}
        <section class="p-6 bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700 opacity-50 pointer-events-none">
          <div class="flex justify-between items-center mb-4">
            <h2 class="text-xl font-semibold text-slate-900 dark:text-white">Preferences</h2>
            <span class="text-xs font-medium px-2 py-1 rounded-full bg-slate-100 dark:bg-slate-700 text-slate-600 dark:text-slate-300">
              Coming Soon
            </span>
          </div>
          <p class="text-sm text-slate-600 dark:text-slate-400">
            Advanced theme settings and default visibility options will be available here.
          </p>
        </section>
      </div>
    </div>
  );
};

export default Settings;
