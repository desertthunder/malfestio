import { api } from "$lib/api";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import DeckView from "./DeckView";

vi.mock("$lib/api", () => ({ api: { get: vi.fn() } }));

vi.mock(
  "@solidjs/router",
  () => ({
    useParams: () => ({ id: "123" }),
    A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a>,
  }),
);

describe("DeckView", () => {
  afterEach(cleanup);

  it("renders deck details and cards", async () => {
    const deck = {
      id: "123",
      title: "Test Deck",
      description: "A test deck",
      tags: ["test"],
      visibility: { type: "Public" },
      owner_did: "did:test",
    };

    const cards = [{ id: "c1", front: "Front 1", back: "Back 1" }, { id: "c2", front: "Front 2", back: "Back 2" }];

    vi.mocked(api.get).mockImplementation(
      ((path: string) => {
        if (path === "/decks/123") {
          return Promise.resolve({ ok: true, json: () => Promise.resolve(deck) });
        }
        if (path === "/decks/123/cards") {
          return Promise.resolve({ ok: true, json: () => Promise.resolve(cards) });
        }
        return Promise.reject(new Error(`Unexpected path: ${path}`));
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      }) as any,
    );

    render(() => <DeckView />);

    await waitFor(() => expect(screen.getByText("Test Deck")).toBeInTheDocument());
    expect(screen.getByText("A test deck")).toBeInTheDocument();
    expect(screen.getByText("#test")).toBeInTheDocument();
    expect(screen.getByText("Front 1")).toBeInTheDocument();
    expect(screen.getByText("Front 2")).toBeInTheDocument();
    expect(screen.getByText("Back 1")).toBeInTheDocument();
  });

  it("renders not found state when deck returns error", async () => {
    vi.mocked(api.get).mockImplementation(
      (() => {
        return Promise.resolve({ ok: false });
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
      }) as any,
    );

    render(() => <DeckView />);

    await waitFor(() => expect(screen.getByText(/Deck not found/i)).toBeInTheDocument());
  });
});
