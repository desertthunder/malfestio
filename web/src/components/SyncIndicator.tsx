import { syncStore } from "$lib/sync-store";
import { Show } from "solid-js";

export function SyncIndicator() {
  const stateClasses = () => {
    const state = syncStore.syncState();
    switch (state) {
      case "syncing":
        return "text-blue-500";
      case "error":
        return "text-red-500";
      case "offline":
        return "text-amber-500";
      default:
        return "text-green-500";
    }
  };

  const stateIcon = () => {
    const state = syncStore.syncState();
    switch (state) {
      case "syncing":
        return "i-ri-loader-4-line animate-spin";
      case "error":
        return "i-ri-error-warning-line";
      case "offline":
        return "i-ri-wifi-off-line";
      default:
        return "i-ri-cloud-line";
    }
  };

  const stateLabel = () => {
    const state = syncStore.syncState();
    switch (state) {
      case "syncing":
        return "Syncing...";
      case "error":
        return "Sync error";
      case "offline":
        return "Offline";
      default:
        return "Synced";
    }
  };

  return (
    <div class="flex items-center gap-2 text-sm">
      <span class={`${stateIcon()} ${stateClasses()}`} />
      <span class={stateClasses()}>{stateLabel()}</span>
      <Show when={syncStore.pendingCount() > 0}>
        <span class="rounded-full bg-blue-100 px-2 py-0.5 text-xs text-blue-700">
          {syncStore.pendingCount()} pending
        </span>
      </Show>
      <Show when={syncStore.conflictCount() > 0}>
        <span class="rounded-full bg-red-100 px-2 py-0.5 text-xs text-red-700">
          {syncStore.conflictCount()} conflicts
        </span>
      </Show>
    </div>
  );
}
