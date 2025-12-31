import { api } from "$lib/api";
import { authStore } from "$lib/store";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import Discovery from "../Discovery";

vi.mock("$lib/api", () => ({ api: { getDiscovery: vi.fn(), getUserProfile: vi.fn() } }));

vi.mock("$lib/store", () => ({ authStore: { user: vi.fn(), isAuthenticated: vi.fn() } }));

vi.mock(
  "@solidjs/router",
  () => ({
    A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a>,
    useNavigate: () => vi.fn(),
  }),
);

describe("Discovery", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockTopTags = { top_tags: [["rust", 10], ["learning", 5]] };

  const mockProfile = {
    did: "did:test:user",
    follower_count: 100,
    following_count: 50,
    deck_count: 10,
    indexed_deck_count: 5,
  };

  it("renders top tags correctly", async () => {
    vi.mocked(api.getDiscovery).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockTopTags) } as unknown as Response,
    );
    vi.mocked(authStore.isAuthenticated).mockReturnValue(false);

    render(() => <Discovery />);

    await waitFor(() => expect(screen.getByText("Discover Malfestio")).toBeInTheDocument());
    await waitFor(() => expect(screen.getByText("#rust")).toBeInTheDocument());
    expect(screen.getByText("10")).toBeInTheDocument();
    expect(screen.getByText("#learning")).toBeInTheDocument();
    expect(screen.getByText("5")).toBeInTheDocument();
  });

  it("shows user profile when logged in", async () => {
    vi.mocked(api.getDiscovery).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockTopTags) } as unknown as Response,
    );
    vi.mocked(authStore.isAuthenticated).mockReturnValue(true);
    vi.mocked(authStore.user).mockReturnValue({ did: "did:test:user", handle: "test.user" });
    vi.mocked(api.getUserProfile).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockProfile) } as unknown as Response,
    );

    render(() => <Discovery />);

    await waitFor(() => expect(screen.getByText("did:test:user")).toBeInTheDocument());
    expect(screen.getByText("100")).toBeInTheDocument();
    expect(screen.getByText("50")).toBeInTheDocument();
    expect(screen.getByText("15")).toBeInTheDocument();
  });

  it("hides user profile when not logged in", async () => {
    vi.mocked(api.getDiscovery).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockTopTags) } as unknown as Response,
    );
    vi.mocked(authStore.isAuthenticated).mockReturnValue(false);

    render(() => <Discovery />);

    await waitFor(() => expect(screen.getByText("Discover Malfestio")).toBeInTheDocument());
    expect(screen.queryByText("did:test:user")).not.toBeInTheDocument();
  });
});
