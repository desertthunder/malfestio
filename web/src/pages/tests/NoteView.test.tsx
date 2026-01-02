import { api } from "$lib/api";
import type { Note } from "$lib/model";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import NoteView from "../NoteView";

vi.mock("$lib/api", () => ({ api: { getNote: vi.fn(), getNotes: vi.fn() } }));

vi.mock(
  "@solidjs/router",
  () => ({
    useParams: () => ({ id: "note-1" }),
    A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a>,
  }),
);

const mockNote: Note = {
  id: "note-1",
  owner_did: "did:plc:test123",
  title: "Test Note Title",
  body: "# Heading\n\nSome **markdown** content with [[Other Note]].",
  tags: ["rust", "learning"],
  visibility: { type: "Public" },
  created_at: "2026-01-01T10:00:00Z",
  updated_at: "2026-01-01T12:00:00Z",
};

const mockAllNotes: Note[] = [mockNote, {
  id: "note-2",
  owner_did: "did:plc:test123",
  title: "Other Note",
  body: "Links back to [[Test Note Title]]",
  tags: [],
  visibility: { type: "Private" },
  created_at: "2026-01-01T10:00:00Z",
  updated_at: "2026-01-01T12:00:00Z",
}];

describe("NoteView", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("renders note title in heading", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
    );
    vi.mocked(api.getNotes).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockAllNotes) } as unknown as Response,
    );

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByRole("heading", { level: 1, name: "Test Note Title" })).toBeInTheDocument();
    });
  });

  it("renders tags", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
    );
    vi.mocked(api.getNotes).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockAllNotes) } as unknown as Response,
    );

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getAllByText("rust").length).toBeGreaterThan(0);
      expect(screen.getAllByText("learning").length).toBeGreaterThan(0);
    });
  });

  it("has back to notes link", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
    );
    vi.mocked(api.getNotes).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockAllNotes) } as unknown as Response,
    );

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "Notes" })).toBeInTheDocument();
    });
  });

  it("renders not found state when note returns error", async () => {
    vi.mocked(api.getNote).mockResolvedValue({ ok: false } as unknown as Response);
    vi.mocked(api.getNotes).mockResolvedValue({ ok: true, json: () => Promise.resolve([]) } as unknown as Response);

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByText("Note not found")).toBeInTheDocument();
    });
  });

  it("renders outline panel heading", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
    );
    vi.mocked(api.getNotes).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockAllNotes) } as unknown as Response,
    );

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByText("Outline")).toBeInTheDocument();
    });
  });

  it("renders wikilinks panel heading", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
    );
    vi.mocked(api.getNotes).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockAllNotes) } as unknown as Response,
    );

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByText("Wikilinks")).toBeInTheDocument();
    });
  });

  it("renders backlinks panel heading", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
    );
    vi.mocked(api.getNotes).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockAllNotes) } as unknown as Response,
    );

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByText("Backlinks")).toBeInTheDocument();
    });
  });
});
