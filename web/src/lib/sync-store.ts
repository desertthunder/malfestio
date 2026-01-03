/**
 * Sync store for managing offline-first sync with PDS.
 */
import { createRoot, createSignal } from "solid-js";
import { api } from "./api";
import { db, generateLocalId, type LocalDeck, type LocalNote, type SyncStatus } from "./db";
import { authStore } from "./store";

export type SyncState = "idle" | "syncing" | "error" | "offline";

function createSyncStore() {
  const [syncState, setSyncState] = createSignal<SyncState>("idle");
  const [pendingCount, setPendingCount] = createSignal(0);
  const [conflictCount, setConflictCount] = createSignal(0);
  const [lastSyncedAt, setLastSyncedAt] = createSignal<string | null>(null);
  const [isOnline, setIsOnline] = createSignal(navigator.onLine);

  if (typeof window !== "undefined") {
    window.addEventListener("online", () => {
      setIsOnline(true);
      processQueue();
    });
    window.addEventListener("offline", () => {
      setIsOnline(false);
      setSyncState("offline");
    });
  }

  async function refreshCounts() {
    const pending = await db.syncQueue.count();
    setPendingCount(pending);

    const conflicts = await db.decks.where("syncStatus").equals("conflict").count()
      + await db.notes.where("syncStatus").equals("conflict").count();
    setConflictCount(conflicts);
  }

  async function saveDeckLocally(
    deck: Omit<LocalDeck, "id" | "syncStatus" | "localVersion" | "updatedAt"> & { id?: string },
  ): Promise<LocalDeck> {
    const now = new Date().toISOString();
    const existing = deck.id ? await db.decks.get(deck.id) : null;

    const localDeck: LocalDeck = {
      id: deck.id || generateLocalId(),
      ownerDid: deck.ownerDid,
      title: deck.title,
      description: deck.description,
      tags: deck.tags,
      visibility: deck.visibility,
      publishedAt: deck.publishedAt,
      forkOf: deck.forkOf,
      syncStatus: existing ? "pending_push" : "local_only",
      localVersion: existing ? existing.localVersion + 1 : 1,
      pdsCid: existing?.pdsCid,
      pdsUri: existing?.pdsUri,
      updatedAt: now,
    };

    await db.decks.put(localDeck);

    if (isOnline()) {
      await queueForSync("deck", localDeck.id, "push");
    }

    await refreshCounts();
    return localDeck;
  }

  async function saveNoteLocally(
    note: Omit<LocalNote, "id" | "syncStatus" | "localVersion" | "updatedAt"> & { id?: string },
  ): Promise<LocalNote> {
    const now = new Date().toISOString();
    const existing = note.id ? await db.notes.get(note.id) : null;

    const localNote: LocalNote = {
      id: note.id || generateLocalId(),
      ownerDid: note.ownerDid,
      title: note.title,
      body: note.body,
      tags: note.tags,
      visibility: note.visibility,
      publishedAt: note.publishedAt,
      links: note.links,
      syncStatus: existing ? "pending_push" : "local_only",
      localVersion: existing ? existing.localVersion + 1 : 1,
      pdsCid: existing?.pdsCid,
      pdsUri: existing?.pdsUri,
      updatedAt: now,
    };

    await db.notes.put(localNote);

    if (isOnline()) {
      await queueForSync("note", localNote.id, "push");
    }

    await refreshCounts();
    return localNote;
  }

  async function queueForSync(entityType: "deck" | "card" | "note", entityId: string, operation: "push" | "delete") {
    const existing = await db.syncQueue.where({ entityType, entityId, operation }).first();

    if (!existing) {
      await db.syncQueue.add({ entityType, entityId, operation, createdAt: new Date().toISOString(), retryCount: 0 });
    }

    await refreshCounts();
  }

  async function processQueue() {
    if (!isOnline() || !authStore.isAuthenticated()) {
      return;
    }

    const items = await db.syncQueue.orderBy("createdAt").toArray();
    if (items.length === 0) return;

    setSyncState("syncing");

    for (const item of items) {
      try {
        if (item.operation === "push") {
          let response: Response;
          if (item.entityType === "deck") {
            response = await api.pushDeck(item.entityId);
          } else if (item.entityType === "note") {
            response = await api.pushNote(item.entityId);
          } else {
            continue;
          }

          if (response.ok) {
            const result = await response.json();
            if (item.entityType === "deck") {
              await db.decks.update(item.entityId, {
                syncStatus: "synced" as SyncStatus,
                pdsCid: result.pds_cid,
                pdsUri: result.pds_uri,
              });
            } else if (item.entityType === "note") {
              await db.notes.update(item.entityId, {
                syncStatus: "synced" as SyncStatus,
                pdsCid: result.pds_cid,
                pdsUri: result.pds_uri,
              });
            }
            await db.syncQueue.delete(item.id!);
          } else if (response.status === 409) {
            if (item.entityType === "deck") {
              await db.decks.update(item.entityId, { syncStatus: "conflict" as SyncStatus });
            } else if (item.entityType === "note") {
              await db.notes.update(item.entityId, { syncStatus: "conflict" as SyncStatus });
            }
            await db.syncQueue.delete(item.id!);
          } else {
            await db.syncQueue.update(item.id!, {
              retryCount: item.retryCount + 1,
              lastError: `HTTP ${response.status}`,
            });
          }
        }
      } catch (error) {
        console.error(`Sync failed for ${item.entityType}:${item.entityId}`, error);
        await db.syncQueue.update(item.id!, {
          retryCount: item.retryCount + 1,
          lastError: error instanceof Error ? error.message : "Unknown error",
        });
      }
    }

    setLastSyncedAt(new Date().toISOString());
    setSyncState(isOnline() ? "idle" : "offline");
    await refreshCounts();
  }

  async function getLocalDecks(ownerDid: string): Promise<LocalDeck[]> {
    return db.decks.where("ownerDid").equals(ownerDid).toArray();
  }
  async function getLocalNotes(ownerDid: string): Promise<LocalNote[]> {
    return db.notes.where("ownerDid").equals(ownerDid).toArray();
  }

  async function getLocalDeck(id: string): Promise<LocalDeck | undefined> {
    return db.decks.get(id);
  }
  async function getLocalNote(id: string): Promise<LocalNote | undefined> {
    return db.notes.get(id);
  }

  async function clearAll() {
    await db.decks.clear();
    await db.cards.clear();
    await db.notes.clear();
    await db.syncQueue.clear();
    await refreshCounts();
  }

  refreshCounts();

  return {
    syncState,
    pendingCount,
    conflictCount,
    lastSyncedAt,
    isOnline,
    saveDeckLocally,
    saveNoteLocally,
    queueForSync,
    processQueue,
    refreshCounts,
    getLocalDecks,
    getLocalNotes,
    getLocalDeck,
    getLocalNote,
    clearAll,
  };
}

export const syncStore = createRoot(createSyncStore);
