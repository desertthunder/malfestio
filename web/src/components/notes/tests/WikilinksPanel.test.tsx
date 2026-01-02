import type { Note } from "$lib/model";
import type { WikiLink } from "$lib/wikilink";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { WikilinksPanel } from "../WikilinksPanel";

vi.mock(
  "@solidjs/router",
  () => ({ A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a> }),
);

describe("WikilinksPanel", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockNotes: Note[] = [{
    id: "note-1",
    owner_did: "did:plc:test",
    title: "Existing Note",
    body: "Content",
    tags: [],
    visibility: { type: "Private" },
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  }];

  const mockLinks: WikiLink[] = [{ raw: "[[Existing Note]]", title: "Existing Note", start: 0, end: 17 }, {
    raw: "[[Missing Note]]",
    title: "Missing Note",
    start: 20,
    end: 36,
  }, { raw: "[[Aliased|Display]]", title: "Aliased", alias: "Display", start: 40, end: 59 }];

  const resolveNote = (title: string) => mockNotes.find((n) => n.title === title) ?? null;

  it("renders heading title", () => {
    render(() => <WikilinksPanel links={[]} notes={[]} resolveNote={() => null} />);
    expect(screen.getByText("Wikilinks")).toBeInTheDocument();
  });

  it("shows empty state when no links", () => {
    render(() => <WikilinksPanel links={[]} notes={[]} resolveNote={() => null} />);
    expect(screen.getByText("No outgoing links")).toBeInTheDocument();
  });

  it("renders resolved links as navigation links", () => {
    render(() => <WikilinksPanel links={mockLinks} notes={mockNotes} resolveNote={resolveNote} />);
    const existingLink = screen.getByRole("link", { name: /Existing Note/ });
    expect(existingLink).toHaveAttribute("href", "/notes/note-1");
  });

  it("renders unresolved links with strikethrough", () => {
    render(() => <WikilinksPanel links={mockLinks} notes={mockNotes} resolveNote={resolveNote} />);
    const missingLink = screen.getByText("Missing Note");
    expect(missingLink).toHaveClass("line-through");
  });

  it("deduplicates links by title", () => {
    const duplicateLinks: WikiLink[] = [{ raw: "[[Same]]", title: "Same", start: 0, end: 8 }, {
      raw: "[[Same]]",
      title: "Same",
      start: 10,
      end: 18,
    }];

    render(() => <WikilinksPanel links={duplicateLinks} notes={[]} resolveNote={() => null} />);
    const sameLinks = screen.getAllByText("Same");
    expect(sameLinks).toHaveLength(1);
  });
});
