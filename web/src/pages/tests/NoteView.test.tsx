import { api } from "$lib/api";
import type { Note } from "$lib/model";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import NoteView from "../NoteView";

vi.mock("$lib/api", () => ({ api: { getNote: vi.fn() } }));

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
  body: "# Heading\n\nSome **markdown** content.",
  tags: ["rust", "learning"],
  visibility: { type: "Public" },
  created_at: "2026-01-01T10:00:00Z",
  updated_at: "2026-01-01T12:00:00Z",
};

describe("NoteView", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("renders note title in heading", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
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

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByText("rust")).toBeInTheDocument();
      expect(screen.getByText("learning")).toBeInTheDocument();
    });
  });

  it("has back to notes link", async () => {
    vi.mocked(api.getNote).mockResolvedValue(
      { ok: true, json: () => Promise.resolve(mockNote) } as unknown as Response,
    );

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByRole("link", { name: "Notes" })).toBeInTheDocument();
    });
  });

  it("renders not found state when note returns error", async () => {
    vi.mocked(api.getNote).mockResolvedValue({ ok: false } as unknown as Response);

    render(() => <NoteView />);

    await waitFor(() => {
      expect(screen.getByText("Note not found")).toBeInTheDocument();
    });
  });
});
