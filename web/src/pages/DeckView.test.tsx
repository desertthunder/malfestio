import { api } from "$lib/api";
import { toast } from "$lib/toast";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import DeckView from "./DeckView";

const { mockNavigate } = vi.hoisted(() => ({ mockNavigate: vi.fn() }));

vi.mock(
  "$lib/api",
  () => ({
    api: { getDeck: vi.fn(), getDeckCards: vi.fn(), forkDeck: vi.fn(), getComments: vi.fn(), addComment: vi.fn() },
  }),
);

vi.mock("$lib/toast", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

vi.mock(
  "@solidjs/router",
  () => ({
    useParams: () => ({ id: "123" }),
    useNavigate: () => mockNavigate,
    A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a>,
  }),
);

describe("DeckView", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockDeck = {
    id: "123",
    title: "Test Deck",
    description: "A test deck",
    tags: ["test"],
    visibility: { type: "Public" },
    owner_did: "did:test",
  };

  const mockCards = [{ id: "c1", front: "Front 1", back: "Back 1" }, { id: "c2", front: "Front 2", back: "Back 2" }];

  it("renders deck details and cards", async () => {
    vi.mocked(api.getDeck).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockDeck) } as unknown as Response,
    );
    vi.mocked(api.getDeckCards).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockCards) } as unknown as Response,
    );
    vi.mocked(api.getComments).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <DeckView />);

    await waitFor(() => expect(screen.getByText("Test Deck")).toBeInTheDocument());
    expect(screen.getByText("A test deck")).toBeInTheDocument();
    expect(screen.getByText("#test")).toBeInTheDocument();
    expect(screen.getByText("Front 1")).toBeInTheDocument();
  });

  it("handles deck fork flow successfully", async () => {
    vi.mocked(api.getDeck).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockDeck) } as unknown as Response,
    );
    vi.mocked(api.getDeckCards).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockCards) } as unknown as Response,
    );
    vi.mocked(api.forkDeck).mockResolvedValue(
      { ok: true, json: () => Promise.resolve({ id: "456" }) } as unknown as Response,
    );
    vi.mocked(api.getComments).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <DeckView />);

    await waitFor(() => expect(screen.getByText("Test Deck")).toBeInTheDocument());

    const forkButton = screen.getByText("Fork Deck", { selector: "button" });
    fireEvent.click(forkButton);

    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText(/Are you sure you want to fork/)).toBeInTheDocument();

    const confirmButton = within(dialog).getByRole("button", { name: /Fork Deck/i });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(api.forkDeck).toHaveBeenCalledWith("123");
      expect(toast.success).toHaveBeenCalledWith("Deck forked successfully!");
      expect(mockNavigate).toHaveBeenCalledWith("/decks/456");
    });
  });

  it("handles deck fork failure", async () => {
    vi.mocked(api.getDeck).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockDeck) } as unknown as Response,
    );
    vi.mocked(api.getDeckCards).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockCards) } as unknown as Response,
    );
    vi.mocked(api.forkDeck).mockResolvedValue({ ok: false } as unknown as Response);
    vi.mocked(api.getComments).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <DeckView />);

    await waitFor(() => expect(screen.getByText("Test Deck")).toBeInTheDocument());

    const forkButton = screen.getByText("Fork Deck", { selector: "button" });
    fireEvent.click(forkButton);

    const dialog = screen.getByRole("dialog");
    const confirmButton = within(dialog).getByRole("button", { name: /Fork Deck/i });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(api.forkDeck).toHaveBeenCalledWith("123");
      expect(toast.error).toHaveBeenCalledWith("Failed to fork deck.");
      expect(mockNavigate).not.toHaveBeenCalled();
    });
  });

  it("renders not found state when deck returns error", async () => {
    vi.mocked(api.getDeck).mockResolvedValue({ ok: false } as unknown as Response);
    render(() => <DeckView />);
    await waitFor(() => expect(screen.getByText(/Deck not found/i)).toBeInTheDocument());
  });
});
