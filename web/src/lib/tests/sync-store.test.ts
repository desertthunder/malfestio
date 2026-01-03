import "fake-indexeddb/auto";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { db, type LocalDeck } from "../db";

vi.mock(
  "../api",
  () => ({
    api: {
      pushDeck: vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ pds_cid: "cid123", pds_uri: "at://test" }),
      }),
      pushNote: vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ pds_cid: "cid456", pds_uri: "at://test" }),
      }),
      getSyncStatus: vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ pending_count: 0, conflict_count: 0 }),
      }),
      resolveConflict: vi.fn().mockResolvedValue({ ok: true }),
    },
  }),
);

vi.mock("../store", () => ({ authStore: { isAuthenticated: () => true, accessJwt: () => "test-token" } }));

import { syncStore } from "../sync-store";

describe("syncStore", () => {
  beforeEach(async () => {
    await db.decks.clear();
    await db.cards.clear();
    await db.notes.clear();
    await db.syncQueue.clear();
  });

  afterEach(async () => {
    await db.decks.clear();
    await db.cards.clear();
    await db.notes.clear();
    await db.syncQueue.clear();
  });

  describe("state signals", () => {
    it("should have initial idle sync state", () => {
      expect(["idle", "offline"]).toContain(syncStore.syncState());
    });

    it("should have online status signal", () => {
      expect(typeof syncStore.isOnline()).toBe("boolean");
    });
  });

  describe("saveDeckLocally", () => {
    it("should save a new deck with local_only status", async () => {
      const deck = await syncStore.saveDeckLocally({
        ownerDid: "did:plc:test",
        title: "New Deck",
        description: "Test description",
        tags: ["test"],
        visibility: { type: "Private" },
      });

      expect(deck.id).toMatch(/^local_/);
      expect(deck.syncStatus).toBe("local_only");
      expect(deck.localVersion).toBe(1);

      const stored = await db.decks.get(deck.id);
      expect(stored?.title).toBe("New Deck");
    });

    it("should update existing deck with pending_push status", async () => {
      const deck = await syncStore.saveDeckLocally({
        ownerDid: "did:plc:test",
        title: "Original Title",
        description: "Test",
        tags: [],
        visibility: { type: "Private" },
      });

      const updated = await syncStore.saveDeckLocally({
        id: deck.id,
        ownerDid: "did:plc:test",
        title: "Updated Title",
        description: "Test",
        tags: [],
        visibility: { type: "Private" },
      });

      expect(updated.id).toBe(deck.id);
      expect(updated.title).toBe("Updated Title");
      expect(updated.syncStatus).toBe("pending_push");
      expect(updated.localVersion).toBe(2);
    });
  });

  describe("saveNoteLocally", () => {
    it("should save a new note with local_only status", async () => {
      const note = await syncStore.saveNoteLocally({
        ownerDid: "did:plc:test",
        title: "New Note",
        body: "Note content",
        tags: ["test"],
        visibility: { type: "Private" },
        links: [],
      });

      expect(note.id).toMatch(/^local_/);
      expect(note.syncStatus).toBe("local_only");
      expect(note.localVersion).toBe(1);
    });
  });

  describe("getLocalDecks", () => {
    it("should return decks for a specific owner", async () => {
      await db.decks.bulkPut([
        {
          id: "deck-1",
          ownerDid: "did:alice",
          title: "Alice Deck",
          description: "",
          tags: [],
          visibility: { type: "Private" },
          syncStatus: "synced",
          localVersion: 1,
          updatedAt: new Date().toISOString(),
        } satisfies LocalDeck,
        {
          id: "deck-2",
          ownerDid: "did:bob",
          title: "Bob Deck",
          description: "",
          tags: [],
          visibility: { type: "Private" },
          syncStatus: "synced",
          localVersion: 1,
          updatedAt: new Date().toISOString(),
        } satisfies LocalDeck,
      ]);

      const aliceDecks = await syncStore.getLocalDecks("did:alice");
      expect(aliceDecks).toHaveLength(1);
      expect(aliceDecks[0].title).toBe("Alice Deck");
    });
  });

  describe("queueForSync", () => {
    it("should add item to sync queue", async () => {
      await syncStore.queueForSync("deck", "deck-123", "push");

      const queue = await db.syncQueue.toArray();
      expect(queue).toHaveLength(1);
      expect(queue[0].entityType).toBe("deck");
      expect(queue[0].entityId).toBe("deck-123");
      expect(queue[0].operation).toBe("push");
    });

    it("should not duplicate queue entries", async () => {
      await syncStore.queueForSync("deck", "deck-123", "push");
      await syncStore.queueForSync("deck", "deck-123", "push");

      const queue = await db.syncQueue.toArray();
      expect(queue).toHaveLength(1);
    });
  });

  describe("refreshCounts", () => {
    it("should update pending and conflict counts", async () => {
      await db.syncQueue.add({
        entityType: "deck",
        entityId: "deck-1",
        operation: "push",
        createdAt: new Date().toISOString(),
        retryCount: 0,
      });

      await db.decks.put({
        id: "deck-2",
        ownerDid: "did:test",
        title: "Conflict Deck",
        description: "",
        tags: [],
        visibility: { type: "Private" },
        syncStatus: "conflict",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      });

      await syncStore.refreshCounts();

      expect(syncStore.pendingCount()).toBe(1);
      expect(syncStore.conflictCount()).toBe(1);
    });
  });

  describe("clearAll", () => {
    it("should clear all local data", async () => {
      await db.decks.put({
        id: "deck-1",
        ownerDid: "did:test",
        title: "Test",
        description: "",
        tags: [],
        visibility: { type: "Private" },
        syncStatus: "synced",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      });

      await syncStore.clearAll();

      expect(await db.decks.count()).toBe(0);
      expect(await db.notes.count()).toBe(0);
      expect(await db.syncQueue.count()).toBe(0);
    });
  });

  describe("saveCardLocally", () => {
    it("should save a new card with local_only status", async () => {
      const card = await syncStore.saveCardLocally({
        deckId: "deck-1",
        front: "Question",
        back: "Answer",
        cardType: "basic",
        hints: [],
      });

      expect(card.id).toMatch(/^local_/);
      expect(card.syncStatus).toBe("local_only");
      expect(card.localVersion).toBe(1);

      const stored = await db.cards.get(card.id);
      expect(stored?.front).toBe("Question");
    });

    it("should update existing card with pending_push status", async () => {
      const card = await syncStore.saveCardLocally({
        deckId: "deck-1",
        front: "Original",
        back: "Answer",
        cardType: "basic",
        hints: [],
      });

      const updated = await syncStore.saveCardLocally({
        id: card.id,
        deckId: "deck-1",
        front: "Updated",
        back: "Answer",
        cardType: "basic",
        hints: [],
      });

      expect(updated.id).toBe(card.id);
      expect(updated.front).toBe("Updated");
      expect(updated.syncStatus).toBe("pending_push");
      expect(updated.localVersion).toBe(2);
    });
  });

  describe("getLocalCards", () => {
    it("should return cards for a specific deck", async () => {
      await syncStore.saveCardLocally({ deckId: "deck-1", front: "Q1", back: "A1", cardType: "basic", hints: [] });
      await syncStore.saveCardLocally({ deckId: "deck-1", front: "Q2", back: "A2", cardType: "basic", hints: [] });
      await syncStore.saveCardLocally({ deckId: "deck-2", front: "Q3", back: "A3", cardType: "basic", hints: [] });

      const cards = await syncStore.getLocalCards("deck-1");
      expect(cards).toHaveLength(2);
      expect(cards.map((c) => c.front)).toContain("Q1");
      expect(cards.map((c) => c.front)).toContain("Q2");
    });
  });

  describe("deleteLocalCard", () => {
    it("should delete a card by id", async () => {
      const card = await syncStore.saveCardLocally({
        deckId: "deck-1",
        front: "To Delete",
        back: "Answer",
        cardType: "basic",
        hints: [],
      });

      await syncStore.deleteLocalCard(card.id);

      const stored = await db.cards.get(card.id);
      expect(stored).toBeUndefined();
    });
  });

  describe("getAllLocalData", () => {
    it("should return all local decks, notes, cards, and queue items", async () => {
      await syncStore.saveDeckLocally({
        ownerDid: "did:test",
        title: "Deck",
        description: "",
        tags: [],
        visibility: { type: "Private" },
      });
      await syncStore.saveNoteLocally({
        ownerDid: "did:test",
        title: "Note",
        body: "Body",
        tags: [],
        visibility: { type: "Private" },
        links: [],
      });
      await syncStore.saveCardLocally({ deckId: "deck-1", front: "Q", back: "A", cardType: "basic", hints: [] });

      const data = await syncStore.getAllLocalData();

      expect(data.decks).toHaveLength(1);
      expect(data.notes).toHaveLength(1);
      expect(data.cards).toHaveLength(1);
      expect(data.queue.length).toBeGreaterThanOrEqual(0);
    });
  });
});
