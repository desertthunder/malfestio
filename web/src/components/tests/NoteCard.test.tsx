import type { Note } from "$lib/model";
import { MemoryRouter, Route } from "@solidjs/router";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NoteCard } from "../NoteCard";

vi.mock("$lib/density-context", () => ({ useDensity: vi.fn(() => "comfortable") }));

const mockNote: Note = {
  id: "note-1",
  owner_did: "did:plc:test123",
  title: "Test Note",
  body: "This is the body of the test note with some **markdown** content.",
  tags: ["rust", "learning"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T10:00:00Z",
  updated_at: "2026-01-01T12:00:00Z",
};

describe("NoteCard", () => {
  afterEach(cleanup);

  it("renders note title", () => {
    render(() => (
      <MemoryRouter>
        <Route path="/" component={() => <NoteCard note={mockNote} />} />
      </MemoryRouter>
    ));
    expect(screen.getByText("Test Note")).toBeInTheDocument();
  });

  it("renders truncated body preview", () => {
    render(() => (
      <MemoryRouter>
        <Route path="/" component={() => <NoteCard note={mockNote} />} />
      </MemoryRouter>
    ));
    expect(screen.getByText(/This is the body/)).toBeInTheDocument();
  });

  it("renders tags", () => {
    render(() => (
      <MemoryRouter>
        <Route path="/" component={() => <NoteCard note={mockNote} />} />
      </MemoryRouter>
    ));
    expect(screen.getByText("rust")).toBeInTheDocument();
    expect(screen.getByText("learning")).toBeInTheDocument();
  });

  it("links to note view page", () => {
    render(() => (
      <MemoryRouter>
        <Route path="/" component={() => <NoteCard note={mockNote} />} />
      </MemoryRouter>
    ));
    const link = screen.getByRole("link");
    expect(link).toHaveAttribute("href", "/notes/note-1");
  });

  it("shows +N for excess tags", () => {
    const noteWithManyTags: Note = { ...mockNote, tags: ["tag1", "tag2", "tag3", "tag4", "tag5"] };
    render(() => (
      <MemoryRouter>
        <Route path="/" component={() => <NoteCard note={noteWithManyTags} />} />
      </MemoryRouter>
    ));
    expect(screen.getByText("+2")).toBeInTheDocument();
  });
});
