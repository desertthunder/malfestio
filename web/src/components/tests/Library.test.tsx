import { api } from "$lib/api";
import Library from "$pages/Library";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import type { JSX } from "solid-js";
import { afterEach, describe, expect, it, type Mock, vi } from "vitest";

vi.mock("$lib/api", () => ({ api: { search: vi.fn() } }));

vi.mock(
  "@solidjs/router",
  () => ({ A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a> }),
);

describe("Library page", () => {
  afterEach(cleanup);

  it("renders loading state initially", () => {
    (api.search as Mock).mockReturnValue(new Promise(() => {}));
    const { container } = render(() => <Library />);
    expect(container.querySelector(".animate-spin")).toBeInTheDocument();
  });

  it("renders remote decks when loaded", async () => {
    (api.search as Mock).mockResolvedValue({
      ok: true,
      json:
        async () => [{
          item_type: "deck",
          item_id: "deck1",
          creator_did: "did:plc:other",
          data: {
            title: "Federated Deck",
            description: "From another server",
            tags: ["remote"],
            at_uri: "at://did:plc:other/org.stormlightlabs.malfestio.deck/deck1",
          },
          rank: 1,
          source: "remote",
        }],
    });

    render(() => <Library />);

    await waitFor(() => expect(screen.getByText("Federated Deck")).toBeInTheDocument());
    expect(screen.getByText("by did:plc:othe...")).toBeInTheDocument();
    expect(screen.getByText("#remote")).toBeInTheDocument();
  });

  it("renders empty state when no content", async () => {
    (api.search as Mock).mockResolvedValue({ ok: true, json: async () => [] });

    render(() => <Library />);

    await waitFor(() => expect(screen.getByText("No federated content found")).toBeInTheDocument());
  });
});
