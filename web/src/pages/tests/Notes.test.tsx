import { api } from "$lib/api";
import type { Note } from "$lib/model";
import { MemoryRouter, Route } from "@solidjs/router";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, type Mock, vi } from "vitest";
import Notes from "../Notes";

vi.mock("$lib/api", () => ({ api: { getNotes: vi.fn() } }));

vi.mock("$lib/density-context", () => ({ useDensity: vi.fn(() => "comfortable") }));

const mockNotes: Note[] = [{
  id: "note-1",
  owner_did: "did:plc:test123",
  title: "First Note",
  body: "Content of first note",
  tags: ["rust"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T10:00:00Z",
  updated_at: "2026-01-01T12:00:00Z",
}, {
  id: "note-2",
  owner_did: "did:plc:test123",
  title: "Second Note",
  body: "Content of second note",
  tags: ["learning"],
  visibility: { type: "Public" },
  created_at: "2026-01-01T11:00:00Z",
  updated_at: "2026-01-01T13:00:00Z",
}];

describe("Notes page", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("renders page header", async () => {
    (api.getNotes as Mock).mockResolvedValue({ ok: true, json: async () => mockNotes });
    render(() => (
      <MemoryRouter>
        <Route path="/" component={Notes} />
      </MemoryRouter>
    ));
    expect(screen.getByRole("heading", { name: "Notes" })).toBeInTheDocument();
    expect(screen.getByText("Your personal knowledge base")).toBeInTheDocument();
  });

  it("renders notes from API", async () => {
    (api.getNotes as Mock).mockResolvedValue({ ok: true, json: async () => mockNotes });
    render(() => (
      <MemoryRouter>
        <Route path="/" component={Notes} />
      </MemoryRouter>
    ));
    await waitFor(() => {
      expect(screen.getByText("First Note")).toBeInTheDocument();
      expect(screen.getByText("Second Note")).toBeInTheDocument();
    });
  });

  it("shows empty state when no notes", async () => {
    (api.getNotes as Mock).mockResolvedValue({ ok: true, json: async () => [] });
    render(() => (
      <MemoryRouter>
        <Route path="/" component={Notes} />
      </MemoryRouter>
    ));
    await waitFor(() => {
      expect(screen.getByText("No notes yet")).toBeInTheDocument();
    });
  });

  it("has New Note button", () => {
    (api.getNotes as Mock).mockResolvedValue({ ok: true, json: async () => [] });
    render(() => (
      <MemoryRouter>
        <Route path="/" component={Notes} />
      </MemoryRouter>
    ));
    expect(screen.getByRole("link", { name: /new note/i })).toBeInTheDocument();
  });

  it("has search input", () => {
    (api.getNotes as Mock).mockResolvedValue({ ok: true, json: async () => [] });
    render(() => (
      <MemoryRouter>
        <Route path="/" component={Notes} />
      </MemoryRouter>
    ));
    expect(screen.getByPlaceholderText("Search notes...")).toBeInTheDocument();
  });
});
