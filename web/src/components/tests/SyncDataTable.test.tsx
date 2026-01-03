import "fake-indexeddb/auto";
import { db } from "$lib/db";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mockSyncStore = vi.hoisted(() => ({
  getAllLocalData: vi.fn(),
  queueForSync: vi.fn(),
  processQueue: vi.fn(),
  clearAll: vi.fn(),
}));

vi.mock("$lib/sync-store", () => ({ syncStore: mockSyncStore }));
vi.mock("$lib/api", () => ({ api: { resolveConflict: vi.fn().mockResolvedValue({ ok: true }) } }));

import { SyncDataTable } from "../SyncDataTable";

describe("SyncDataTable", () => {
  beforeEach(async () => {
    vi.clearAllMocks();
    mockSyncStore.getAllLocalData.mockResolvedValue({ decks: [], notes: [], cards: [], queue: [] });
    await db.decks.clear();
    await db.notes.clear();
    await db.cards.clear();
    await db.syncQueue.clear();
  });

  afterEach(cleanup);

  it("renders with empty state", async () => {
    render(() => <SyncDataTable />);

    await new Promise((r) => setTimeout(r, 100));

    expect(screen.getByText("Records (0)")).toBeInTheDocument();
    expect(screen.getByText("Queue (0)")).toBeInTheDocument();
  });

  it("renders records tab with data", async () => {
    mockSyncStore.getAllLocalData.mockResolvedValue({
      decks: [{
        id: "deck-1",
        title: "Test Deck",
        syncStatus: "synced",
        localVersion: 1,
        updatedAt: new Date().toISOString(),
      }],
      notes: [],
      cards: [],
      queue: [],
    });

    render(() => <SyncDataTable />);

    await new Promise((r) => setTimeout(r, 100));

    expect(screen.getByText("Records (1)")).toBeInTheDocument();
  });

  it("renders queue tab with pending items", async () => {
    mockSyncStore.getAllLocalData.mockResolvedValue({
      decks: [],
      notes: [],
      cards: [],
      queue: [{
        id: 1,
        entityType: "deck",
        entityId: "deck-1",
        operation: "push",
        retryCount: 0,
        createdAt: new Date().toISOString(),
      }],
    });

    render(() => <SyncDataTable />);

    await new Promise((r) => setTimeout(r, 100));

    expect(screen.getByText("Queue (1)")).toBeInTheDocument();
  });

  it("has refresh and clear buttons", async () => {
    render(() => <SyncDataTable />);

    await new Promise((r) => setTimeout(r, 100));

    expect(screen.getByText("Refresh")).toBeInTheDocument();
    expect(screen.getByText("Clear All")).toBeInTheDocument();
  });
});
