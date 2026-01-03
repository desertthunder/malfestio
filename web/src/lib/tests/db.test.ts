import "fake-indexeddb/auto";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { db, generateLocalId, isLocalId, type LocalDeck, type LocalNote } from "../db";

describe("db", () => {
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

  describe("generateLocalId", () => {
    it("should generate unique IDs with local_ prefix", () => {
      const id1 = generateLocalId();
      const id2 = generateLocalId();

      expect(id1).toMatch(/^local_\d+_[a-z0-9]+$/);
      expect(id2).toMatch(/^local_\d+_[a-z0-9]+$/);
      expect(id1).not.toBe(id2);
    });
  });

  describe("isLocalId", () => {
    it("should return true for local IDs", () => {
      expect(isLocalId("local_123_abc")).toBe(true);
      expect(isLocalId("local_")).toBe(true);
    });

    it("should return false for server IDs", () => {
      expect(isLocalId("deck-123")).toBe(false);
      expect(isLocalId("uuid-abc-def")).toBe(false);
    });
  });

  describe("decks table", () => {
    it("should insert and retrieve decks", async () => {
      const deck: LocalDeck = {
        id: "deck-1",
        ownerDid: "did:plc:test",
        title: "Test Deck",
        description: "A test deck",
        tags: ["test"],
        visibility: { type: "Private" },
        syncStatus: "local_only",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      };

      await db.decks.put(deck);
      const retrieved = await db.decks.get("deck-1");

      expect(retrieved).toBeDefined();
      expect(retrieved?.title).toBe("Test Deck");
      expect(retrieved?.syncStatus).toBe("local_only");
    });

    it("should query decks by ownerDid", async () => {
      await db.decks.bulkPut([{
        id: "deck-1",
        ownerDid: "did:alice",
        title: "Alice Deck 1",
        description: "",
        tags: [],
        visibility: { type: "Private" },
        syncStatus: "synced",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      }, {
        id: "deck-2",
        ownerDid: "did:alice",
        title: "Alice Deck 2",
        description: "",
        tags: [],
        visibility: { type: "Public" },
        syncStatus: "pending_push",
        localVersion: 2,
        updatedAt: new Date().toISOString(),
      }, {
        id: "deck-3",
        ownerDid: "did:bob",
        title: "Bob Deck",
        description: "",
        tags: [],
        visibility: { type: "Private" },
        syncStatus: "local_only",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      }]);

      const aliceDecks = await db.decks.where("ownerDid").equals("did:alice").toArray();
      expect(aliceDecks).toHaveLength(2);
      expect(aliceDecks.map((d) => d.title)).toContain("Alice Deck 1");
      expect(aliceDecks.map((d) => d.title)).toContain("Alice Deck 2");
    });

    it("should query decks by syncStatus", async () => {
      await db.decks.bulkPut([{
        id: "deck-1",
        ownerDid: "did:test",
        title: "Synced",
        description: "",
        tags: [],
        visibility: { type: "Private" },
        syncStatus: "synced",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      }, {
        id: "deck-2",
        ownerDid: "did:test",
        title: "Conflict",
        description: "",
        tags: [],
        visibility: { type: "Private" },
        syncStatus: "conflict",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      }]);

      const conflicts = await db.decks.where("syncStatus").equals("conflict").toArray();
      expect(conflicts).toHaveLength(1);
      expect(conflicts[0].title).toBe("Conflict");
    });
  });

  describe("notes table", () => {
    it("should insert and retrieve notes", async () => {
      const note: LocalNote = {
        id: "note-1",
        ownerDid: "did:plc:test",
        title: "Test Note",
        body: "This is a test note",
        tags: ["test"],
        visibility: { type: "Private" },
        links: [],
        syncStatus: "local_only",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      };

      await db.notes.put(note);
      const retrieved = await db.notes.get("note-1");

      expect(retrieved).toBeDefined();
      expect(retrieved?.title).toBe("Test Note");
      expect(retrieved?.body).toBe("This is a test note");
    });
  });

  describe("syncQueue table", () => {
    it("should auto-increment IDs", async () => {
      const id1 = await db.syncQueue.add({
        entityType: "deck",
        entityId: "deck-1",
        operation: "push",
        createdAt: new Date().toISOString(),
        retryCount: 0,
      });

      const id2 = await db.syncQueue.add({
        entityType: "note",
        entityId: "note-1",
        operation: "push",
        createdAt: new Date().toISOString(),
        retryCount: 0,
      });

      expect(id2).toBeGreaterThan(id1!);
    });

    it("should count pending items", async () => {
      await db.syncQueue.bulkAdd([{
        entityType: "deck",
        entityId: "deck-1",
        operation: "push",
        createdAt: new Date().toISOString(),
        retryCount: 0,
      }, {
        entityType: "deck",
        entityId: "deck-2",
        operation: "push",
        createdAt: new Date().toISOString(),
        retryCount: 0,
      }, {
        entityType: "note",
        entityId: "note-1",
        operation: "push",
        createdAt: new Date().toISOString(),
        retryCount: 0,
      }]);

      const count = await db.syncQueue.count();
      expect(count).toBe(3);
    });
  });
});
