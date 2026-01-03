import "fake-indexeddb/auto";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// Create a controllable mock store
const mockState = { state: "idle" as string, pending: 0, conflicts: 0 };

vi.mock(
  "$lib/sync-store",
  () => ({
    syncStore: {
      syncState: () => mockState.state,
      pendingCount: () => mockState.pending,
      conflictCount: () => mockState.conflicts,
    },
  }),
);

import { SyncIndicator } from "../SyncIndicator";

describe("SyncIndicator", () => {
  beforeEach(() => {
    mockState.state = "idle";
    mockState.pending = 0;
    mockState.conflicts = 0;
  });

  afterEach(cleanup);

  it("renders synced state by default", () => {
    render(() => <SyncIndicator />);
    expect(screen.getByText("Synced")).toBeInTheDocument();
  });

  it("renders syncing state with spinner", () => {
    mockState.state = "syncing";
    render(() => <SyncIndicator />);
    expect(screen.getByText("Syncing...")).toBeInTheDocument();
  });

  it("renders error state", () => {
    mockState.state = "error";
    render(() => <SyncIndicator />);
    expect(screen.getByText("Sync error")).toBeInTheDocument();
  });

  it("renders offline state", () => {
    mockState.state = "offline";
    render(() => <SyncIndicator />);
    expect(screen.getByText("Offline")).toBeInTheDocument();
  });

  it("shows pending count when items are pending", () => {
    mockState.pending = 3;
    render(() => <SyncIndicator />);
    expect(screen.getByText("3 pending")).toBeInTheDocument();
  });

  it("shows conflict count when conflicts exist", () => {
    mockState.conflicts = 2;
    render(() => <SyncIndicator />);
    expect(screen.getByText("2 conflicts")).toBeInTheDocument();
  });

  it("hides pending badge when count is zero", () => {
    mockState.pending = 0;
    render(() => <SyncIndicator />);
    expect(screen.queryByText(/pending/)).not.toBeInTheDocument();
  });

  it("hides conflict badge when count is zero", () => {
    mockState.conflicts = 0;
    render(() => <SyncIndicator />);
    expect(screen.queryByText(/conflicts/)).not.toBeInTheDocument();
  });
});
