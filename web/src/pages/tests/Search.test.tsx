import { api } from "$lib/api";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import Search from "../Search";

vi.mock("$lib/api", () => ({ api: { search: vi.fn() } }));

const { mockSearchParams } = vi.hoisted(() => ({ mockSearchParams: { q: "" } }));

vi.mock(
  "@solidjs/router",
  () => ({
    useSearchParams: () => [mockSearchParams],
    A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a>,
    useNavigate: () => vi.fn(),
  }),
);

describe("Search", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockSearchResults = [{
    item_type: "deck",
    item_id: "deck1",
    creator_did: "did:test:1",
    data: {
      id: "deck1",
      owner_did: "did:test:1",
      title: "Test Deck",
      description: "A test deck",
      tags: ["test"],
      visibility: { type: "Public" },
    },
    rank: 0.9,
  }, {
    item_type: "card",
    item_id: "card1",
    creator_did: "did:test:1",
    data: { id: "card1", deck_id: "deck1", front: "Card Front", back: "Card Back", owner_did: "did:test:1" },
    rank: 0.8,
  }, {
    item_type: "note",
    item_id: "note1",
    creator_did: "did:test:1",
    data: { id: "note1", title: "Test Note", owner_did: "did:test:1" },
    rank: 0.7,
  }];

  it("renders search results correctly", async () => {
    mockSearchParams.q = "test";
    vi.mocked(api.search).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockSearchResults) } as unknown as Response,
    );

    render(() => <Search />);

    // Verify loading state or results
    await waitFor(() => expect(screen.getByText("Search Results")).toBeInTheDocument());

    // Verify Deck result
    await waitFor(() => expect(screen.getByText("Test Deck")).toBeInTheDocument());
    expect(screen.getByText("A test deck")).toBeInTheDocument();

    // Verify Card result
    expect(screen.getByText("Card Front")).toBeInTheDocument();
    expect(screen.getByText("Card Back")).toBeInTheDocument();

    // Verify Note result
    expect(screen.getByText("Test Note")).toBeInTheDocument();
  });

  it("shows empty state when no results", async () => {
    mockSearchParams.q = "nonexistent";
    vi.mocked(api.search).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <Search />);

    await waitFor(() => expect(screen.getByText("No results found for \"nonexistent\"")).toBeInTheDocument());
  });

  it("handles loading state", async () => {
    mockSearchParams.q = "loading";
    vi.mocked(api.search).mockReturnValue(new Promise(() => {}));

    render(() => <Search />);

    expect(api.search).toHaveBeenCalledWith("loading");
  });

  it("generates correct links for results", async () => {
    mockSearchParams.q = "test";
    vi.mocked(api.search).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockSearchResults) } as unknown as Response,
    );

    render(() => <Search />);

    await waitFor(() => expect(screen.getByText("Test Deck")).toBeInTheDocument());

    const deckLink = screen.getByText("Test Deck").closest("a");
    expect(deckLink).toHaveAttribute("href", "/decks/deck1");

    const cardLink = screen.getByText("Card in Deck").closest("a");
    expect(cardLink).toHaveAttribute("href", "/decks/deck1");

    const noteLink = screen.getByText("Test Note").closest("a");
    expect(noteLink).toHaveAttribute("href", "/notes/note1");
  });
});
