import { api } from "$lib/api";
import type { LocalCard, LocalDeck, LocalNote, SyncQueueItem } from "$lib/db";
import { syncStore } from "$lib/sync-store";
import type { Column } from "$ui/DataTable";
import { DataTable } from "$ui/DataTable";
import { createResource, createSignal, Show } from "solid-js";

type SyncRecord = {
  id: string;
  type: "deck" | "note" | "card";
  title: string;
  status: string;
  version: number;
  updatedAt: string;
};

type QueueRecord = {
  id: string;
  entityType: string;
  entityId: string;
  operation: string;
  retryCount: number;
  createdAt: string;
  lastError?: string;
};

export function SyncDataTable() {
  const [activeTab, setActiveTab] = createSignal<"records" | "queue">("records");
  const [refreshKey, setRefreshKey] = createSignal(0);

  const [data, { refetch }] = createResource(refreshKey, async () => {
    const result = await syncStore.getAllLocalData();
    return result;
  });

  const syncRecords = (): SyncRecord[] => {
    const d = data();
    if (!d) return [];

    const decks: SyncRecord[] = d.decks.map((deck: LocalDeck) => ({
      id: deck.id,
      type: "deck" as const,
      title: deck.title,
      status: deck.syncStatus,
      version: deck.localVersion,
      updatedAt: deck.updatedAt,
    }));

    const notes: SyncRecord[] = d.notes.map((note: LocalNote) => ({
      id: note.id,
      type: "note" as const,
      title: note.title,
      status: note.syncStatus,
      version: note.localVersion,
      updatedAt: note.updatedAt,
    }));

    const cards: SyncRecord[] = d.cards.map((card: LocalCard) => ({
      id: card.id,
      type: "card" as const,
      title: card.front.slice(0, 50) + (card.front.length > 50 ? "..." : ""),
      status: card.syncStatus,
      version: card.localVersion,
      updatedAt: "",
    }));

    return [...decks, ...notes, ...cards];
  };

  const queueRecords = (): QueueRecord[] => {
    const d = data();
    if (!d) return [];

    return d.queue.map((item: SyncQueueItem) => ({
      id: String(item.id || ""),
      entityType: item.entityType,
      entityId: item.entityId,
      operation: item.operation,
      retryCount: item.retryCount,
      createdAt: item.createdAt,
      lastError: item.lastError,
    }));
  };

  const handleSync = async (type: string, id: string) => {
    await syncStore.queueForSync(type as "deck" | "card" | "note", id, "push");
    await syncStore.processQueue();
    setRefreshKey((k) => k + 1);
  };

  const handleResolve = async (
    type: string,
    id: string,
    strategy: "last_write_wins" | "keep_local" | "keep_remote",
  ) => {
    await api.resolveConflict(type, id, strategy);
    setRefreshKey((k) => k + 1);
  };

  const handleClear = async () => {
    if (confirm("Clear all local sync data? This cannot be undone.")) {
      await syncStore.clearAll();
      setRefreshKey((k) => k + 1);
    }
  };

  const recordColumns: Column<SyncRecord>[] = [
    { key: "type", header: "Type", sortable: true, width: "80px" },
    { key: "title", header: "Title", sortable: true },
    {
      key: "status",
      header: "Status",
      sortable: true,
      width: "120px",
      render: (row) => (
        <span
          class={`px-2 py-1 rounded text-xs ${
            row.status === "synced"
              ? "bg-green-900 text-green-300"
              : row.status === "conflict"
              ? "bg-red-900 text-red-300"
              : row.status === "pending_push"
              ? "bg-blue-900 text-blue-300"
              : "bg-gray-700 text-gray-300"
          }`}>
          {row.status}
        </span>
      ),
    },
    { key: "version", header: "Ver", sortable: true, width: "60px" },
    {
      key: "actions",
      header: "Actions",
      width: "150px",
      render: (row) => (
        <div class="flex gap-2">
          <Show when={row.status === "conflict"}>
            <button
              onClick={() => handleResolve(row.type, row.id, "keep_local")}
              class="text-xs text-blue-400 hover:underline">
              Keep Local
            </button>
          </Show>
          <Show when={row.status !== "synced"}>
            <button onClick={() => handleSync(row.type, row.id)} class="text-xs text-green-400 hover:underline">
              Sync
            </button>
          </Show>
        </div>
      ),
    },
  ];

  const queueColumns: Column<QueueRecord>[] = [
    { key: "entityType", header: "Type", sortable: true, width: "80px" },
    { key: "entityId", header: "Entity ID", sortable: true },
    { key: "operation", header: "Op", width: "60px" },
    { key: "retryCount", header: "Retries", sortable: true, width: "80px" },
    { key: "lastError", header: "Error" },
  ];

  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex gap-2">
          <button
            onClick={() => setActiveTab("records")}
            class={`px-3 py-1.5 text-sm rounded ${
              activeTab() === "records" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-300"
            }`}>
            Records ({syncRecords().length})
          </button>
          <button
            onClick={() => setActiveTab("queue")}
            class={`px-3 py-1.5 text-sm rounded ${
              activeTab() === "queue" ? "bg-blue-600 text-white" : "bg-gray-700 text-gray-300"
            }`}>
            Queue ({queueRecords().length})
          </button>
        </div>
        <div class="flex gap-2">
          <button
            onClick={() => refetch()}
            class="px-3 py-1.5 text-sm bg-gray-700 text-gray-300 rounded hover:bg-gray-600">
            Refresh
          </button>
          <button onClick={handleClear} class="px-3 py-1.5 text-sm bg-red-900 text-red-300 rounded hover:bg-red-800">
            Clear All
          </button>
        </div>
      </div>

      <Show when={data.loading}>
        <div class="text-gray-400 text-sm">Loading...</div>
      </Show>

      <Show when={!data.loading && activeTab() === "records"}>
        <Show when={syncRecords().length > 0} fallback={<div class="text-gray-500 text-sm">No local records</div>}>
          <DataTable columns={recordColumns} data={syncRecords()} getRowId={(r) => r.id} />
        </Show>
      </Show>

      <Show when={!data.loading && activeTab() === "queue"}>
        <Show
          when={queueRecords().length > 0}
          fallback={<div class="text-gray-500 text-sm">No pending queue items</div>}>
          <DataTable columns={queueColumns} data={queueRecords()} getRowId={(r) => r.id} />
        </Show>
      </Show>
    </div>
  );
}
