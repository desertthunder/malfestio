import { api } from "$lib/api";
import DeckPreview from "$pages/DeckPreview";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import type { JSX } from "solid-js";
import { afterEach, describe, expect, it, type Mock, vi } from "vitest";

vi.mock("$lib/api", () => ({ api: { getRemoteDeck: vi.fn() } }));

vi.mock(
  "@solidjs/router",
  () => ({
    useSearchParams: () => [{ uri: "at://did:plc:test/org.stormlightlabs.malfestio.deck/123" }],
    useNavigate: () => vi.fn(),
    A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a>,
  }),
);

describe("DeckPreview", () => {
  afterEach(cleanup);

  it("renders loading state initially", () => {
    (api.getRemoteDeck as Mock).mockReturnValue(new Promise(() => {}));
    const { container } = render(() => <DeckPreview />);
    expect(container.querySelector(".animate-spin")).toBeInTheDocument();
  });

  it("renders deck details when loaded", async () => {
    (api.getRemoteDeck as Mock).mockResolvedValue({
      ok: true,
      json: async () => ({
        deck: {
          id: "at://did:plc:test/org.stormlightlabs.malfestio.deck/123",
          owner_did: "did:plc:test",
          title: "Remote Deck",
          description: "A test deck",
          tags: ["test"],
          visibility: { type: "Public" },
        },
        cards: [{ id: "card1", front: "Question", back: "Answer", deck_id: "deck1", owner_did: "did:plc:test" }],
      }),
    });

    render(() => <DeckPreview />);

    await waitFor(() => expect(screen.getByText("Remote Deck")).toBeInTheDocument());
    expect(screen.getByText("By did:plc:test")).toBeInTheDocument();
    expect(screen.getByText("Question")).toBeInTheDocument();
    expect(screen.getByText("Answer")).toBeInTheDocument();
  });

  it("renders error state when fetch fails", async () => {
    (api.getRemoteDeck as Mock).mockResolvedValue({ ok: false });

    render(() => <DeckPreview />);

    await waitFor(() => expect(screen.getByText("Could not load the requested remote deck.")).toBeInTheDocument());
  });
});
