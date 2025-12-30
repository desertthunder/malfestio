import { api } from "$lib/api";
import { toast } from "$lib/toast";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import Feed from "../Feed";

const { mockNavigate } = vi.hoisted(() => ({ mockNavigate: vi.fn() }));

vi.mock(
  "$lib/api",
  () => ({
    api: {
      getFeedFollows: vi.fn(),
      getFeedTrending: vi.fn(),
      forkDeck: vi.fn(),
      follow: vi.fn(),
      unfollow: vi.fn(),
      getFollowers: vi.fn(),
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

describe("Feed", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockDecks = [{
    id: "deck1",
    title: "Test Deck 1",
    description: "A test deck",
    tags: ["test"],
    visibility: { type: "Public" },
    owner_did: "did:test:1",
    published_at: "2024-01-01T00:00:00Z",
  }, {
    id: "deck2",
    title: "Test Deck 2",
    description: "Another test deck",
    tags: ["demo"],
    visibility: { type: "Public" },
    owner_did: "did:test:2",
    published_at: null,
  }];

  it("renders feed with decks from followed users", async () => {
    vi.mocked(api.getFeedFollows).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockDecks) } as unknown as Response,
    );
    vi.mocked(api.getFeedTrending).mockResolvedValue(
      { ok: true, json: () => Promise.resolve([]) } as unknown as Response,
    );
    vi.mocked(api.getFollowers).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <Feed />);

    await waitFor(() => expect(screen.getByText("Test Deck 1")).toBeInTheDocument());
    expect(screen.getByText("Test Deck 2")).toBeInTheDocument();
  });

  it("shows empty state when no followed decks", async () => {
    vi.mocked(api.getFeedFollows).mockResolvedValue(
      { ok: true, json: () => Promise.resolve([]) } as unknown as Response,
    );
    vi.mocked(api.getFeedTrending).mockResolvedValue(
      { ok: true, json: () => Promise.resolve([]) } as unknown as Response,
    );

    render(() => <Feed />);

    await waitFor(() => expect(screen.getByText(/No updates from followed users/i)).toBeInTheDocument());
  });

  it("handles fork flow successfully", async () => {
    vi.mocked(api.getFeedFollows).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockDecks) } as unknown as Response,
    );
    vi.mocked(api.getFeedTrending).mockResolvedValue(
      { ok: true, json: () => Promise.resolve([]) } as unknown as Response,
    );
    vi.mocked(api.forkDeck).mockResolvedValue(
      { ok: true, json: () => Promise.resolve({ id: "forked-deck" }) } as unknown as Response,
    );
    vi.mocked(api.getFollowers).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <Feed />);

    await waitFor(() => expect(screen.getByText("Test Deck 1")).toBeInTheDocument());

    const forkButtons = screen.getAllByText("Fork");
    fireEvent.click(forkButtons[0]);

    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText(/Are you sure you want to fork/i)).toBeInTheDocument();

    const confirmButton = within(dialog).getByRole("button", { name: /Fork Deck/i });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(api.forkDeck).toHaveBeenCalledWith("deck1");
      expect(toast.success).toHaveBeenCalledWith("Deck forked successfully!");
      expect(mockNavigate).toHaveBeenCalledWith("/decks/forked-deck");
    });
  });

  it("handles fork failure", async () => {
    vi.mocked(api.getFeedFollows).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockDecks) } as unknown as Response,
    );
    vi.mocked(api.getFeedTrending).mockResolvedValue(
      { ok: true, json: () => Promise.resolve([]) } as unknown as Response,
    );
    vi.mocked(api.forkDeck).mockResolvedValue({ ok: false } as unknown as Response);
    vi.mocked(api.getFollowers).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <Feed />);

    await waitFor(() => expect(screen.getByText("Test Deck 1")).toBeInTheDocument());

    const forkButtons = screen.getAllByText("Fork");
    fireEvent.click(forkButtons[0]);

    const dialog = screen.getByRole("dialog");
    const confirmButton = within(dialog).getByRole("button", { name: /Fork Deck/i });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(api.forkDeck).toHaveBeenCalledWith("deck1");
      expect(toast.error).toHaveBeenCalledWith("Failed to fork deck.");
      expect(mockNavigate).not.toHaveBeenCalled();
    });
  });
});
