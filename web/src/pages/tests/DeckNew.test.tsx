import "fake-indexeddb/auto";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import DeckNew from "../DeckNew";

const { mockNavigate } = vi.hoisted(() => ({ mockNavigate: vi.fn() }));

vi.mock("$lib/api", () => ({ api: { createDeck: vi.fn(), updatePreferences: vi.fn() } }));

vi.mock(
  "$lib/store",
  () => ({
    prefStore: { prefs: vi.fn(() => ({ tutorial_deck_completed: true })), fetchPrefs: vi.fn() },
    authStore: { user: vi.fn(() => ({ did: "did:plc:test" })) },
  }),
);

vi.mock(
  "$lib/sync-store",
  () => ({
    syncStore: {
      saveDeckLocally: vi.fn().mockResolvedValue({ id: "local_123" }),
      saveCardLocally: vi.fn().mockResolvedValue({ id: "card_123" }),
      isOnline: vi.fn(() => true),
    },
  }),
);

vi.mock("$lib/toast", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

vi.mock(
  "@solidjs/router",
  () => ({
    useNavigate: () => mockNavigate,
    A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a>,
  }),
);

vi.mock(
  "$components/DeckEditor",
  () => ({
    DeckEditor: (props: { onSave: (data: unknown) => void }) => (
      <div data-testid="deck-editor">
        <button
          onClick={() =>
            props.onSave({ title: "Test", description: "", tags: [], visibility: { type: "Private" }, cards: [] })}>
          Save
        </button>
      </div>
    ),
  }),
);

vi.mock("$components/TutorialOverlay", () => ({ TutorialOverlay: () => <div data-testid="tutorial-overlay" /> }));

vi.mock(
  "$lib/TutorialProvider",
  () => ({
    TutorialProvider: (props: { children: JSX.Element }) => <>{props.children}</>,
    useTutorial: () => ({ active: () => false, shouldShowTutorial: () => false, startTutorial: vi.fn() }),
  }),
);

describe("DeckNew", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("renders page title", () => {
    render(() => <DeckNew />);
    expect(screen.getByText("Create New Deck")).toBeInTheDocument();
  });

  it("renders page description", () => {
    render(() => <DeckNew />);
    expect(screen.getByText("Start a new collection of flashcards.")).toBeInTheDocument();
  });

  it("renders DeckEditor component", () => {
    render(() => <DeckNew />);
    expect(screen.getByTestId("deck-editor")).toBeInTheDocument();
  });

  it("renders TutorialOverlay component", () => {
    render(() => <DeckNew />);
    expect(screen.getByTestId("tutorial-overlay")).toBeInTheDocument();
  });

  it("shows tutorial button when tutorial already completed", () => {
    render(() => <DeckNew />);
    expect(screen.getByText(/Show Tutorial/i)).toBeInTheDocument();
  });
});
