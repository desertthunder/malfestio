import { api } from "$lib/api";
import type { DensityMode } from "$lib/design-tokens";
import { prefStore } from "$lib/store";
import type { Component } from "solid-js";
import { createSignal, For, Show } from "solid-js";
import { SyncDataTable } from "../components/SyncDataTable";

type DensityOption = { value: DensityMode; label: string; description: string };

// TODO: move to constants
const densityOptions: DensityOption[] = [
  { value: "compact", label: "Compact", description: "Minimal spacing, more content at a glance" },
  { value: "comfortable", label: "Comfortable", description: "Balanced spacing for everyday use" },
  { value: "spacious", label: "Spacious", description: "Generous spacing, easier on the eyes" },
];

const Settings: Component = () => {
  const [exportingDecks, setExportingDecks] = createSignal(false);
  const [exportingNotes, setExportingNotes] = createSignal(false);
  const [exportError, setExportError] = createSignal<string | null>(null);
  const [savingDensity, setSavingDensity] = createSignal(false);

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

  const handleDensityChange = async (mode: DensityMode) => {
    setSavingDensity(true);
    await prefStore.updatePreferences({ density_mode: mode });
    setSavingDensity(false);
  };

  const currentDensity = () => prefStore.densityMode() as DensityMode;

  return (
    <div class="max-w-4xl mx-auto p-6 space-y-8">
      <header class="space-y-4">
        <h1 class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white">Settings</h1>
        <p class="text-slate-600 dark:text-slate-400">Manage your account preferences and data.</p>
      </header>

      <div class="grid gap-6">
        <section class="p-6 bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700">
          <h2 class="text-xl font-semibold text-slate-900 dark:text-white mb-4">Interface Density</h2>
          <p class="text-sm text-slate-600 dark:text-slate-400 mb-6">
            Control how much information is displayed at once. Compact shows more content, spacious provides more
            breathing room.
          </p>

          <div class="grid gap-3 sm:grid-cols-3">
            <For each={densityOptions}>
              {(option) => (
                <button
                  type="button"
                  disabled={savingDensity()}
                  onClick={() => handleDensityChange(option.value)}
                  class={`
                    p-4 rounded-lg border-2 text-left transition-all
                    ${
                    currentDensity() === option.value
                      ? "border-blue-500 bg-blue-50 dark:bg-blue-900/20"
                      : "border-slate-200 dark:border-slate-700 hover:border-slate-300 dark:hover:border-slate-600"
                  }
                    ${savingDensity() ? "opacity-50 cursor-wait" : "cursor-pointer"}
                  `}>
                  <div class="flex items-center gap-2 mb-1">
                    <span
                      class={`w-4 h-4 rounded-full border-2 flex items-center justify-center ${
                        currentDensity() === option.value
                          ? "border-blue-500 bg-blue-500"
                          : "border-slate-400 dark:border-slate-500"
                      }`}>
                      <Show when={currentDensity() === option.value}>
                        <span class="w-2 h-2 rounded-full bg-white" />
                      </Show>
                    </span>
                    <span class="font-medium text-slate-900 dark:text-white">{option.label}</span>
                  </div>
                  <p class="text-xs text-slate-500 dark:text-slate-400 ml-6">{option.description}</p>
                </button>
              )}
            </For>
          </div>
        </section>

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

        <section class="p-6 bg-white dark:bg-slate-800 rounded-xl shadow-sm border border-slate-200 dark:border-slate-700">
          <h2 class="text-xl font-semibold text-slate-900 dark:text-white mb-4">Local Sync Data</h2>
          <p class="text-sm text-slate-600 dark:text-slate-400 mb-6">
            View and manage locally cached data and pending sync operations.
          </p>
          <SyncDataTable />
        </section>
      </div>
    </div>
  );
};

export default Settings;
