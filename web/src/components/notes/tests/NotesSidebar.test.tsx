import type { Note } from "$lib/model";
import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import type { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NotesSidebar } from "../NotesSidebar";

vi.mock(
  "@solidjs/router",
  () => ({
    A: (props: { href: string; class: string; children: JSX.Element; "data-testid"?: string }) => (
      <a href={props.href} class={props.class} data-testid={props["data-testid"]}>{props.children}</a>
    ),
  }),
);

const mockNotes: Note[] = [{
  id: "note-1",
  owner_did: "did:plc:test",
  title: "First Note",
  body: "Content with [[Second Note]] link",
  tags: ["rust", "learning"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-03T00:00:00Z",
}, {
  id: "note-2",
  owner_did: "did:plc:test",
  title: "Second Note",
  body: "This links to [[First Note]]",
  tags: ["rust", "advanced"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-02T00:00:00Z",
}, {
  id: "note-3",
  owner_did: "did:plc:test",
  title: "Orphan Note",
  body: "No wikilinks here",
  tags: ["learning"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}];

describe("NotesSidebar", () => {
  afterEach(cleanup);

  it("renders the sidebar", () => {
    render(() => <NotesSidebar notes={mockNotes} />);
    expect(screen.getByTestId("notes-sidebar")).toBeInTheDocument();
  });

  it("displays filter buttons with counts", () => {
    render(() => <NotesSidebar notes={mockNotes} />);
    expect(screen.getByTestId("filter-all")).toHaveTextContent("All (3)");
    expect(screen.getByTestId("filter-linked")).toHaveTextContent("Linked (2)");
    expect(screen.getByTestId("filter-orphaned")).toHaveTextContent("Orphaned (1)");
  });

  it("displays unique tags with correct counts", () => {
    render(() => <NotesSidebar notes={mockNotes} />);

    expect(screen.getByTestId("tag-rust")).toHaveTextContent("#rust");
    expect(screen.getByTestId("tag-rust")).toHaveTextContent("2");

    expect(screen.getByTestId("tag-learning")).toHaveTextContent("#learning");
    expect(screen.getByTestId("tag-learning")).toHaveTextContent("2");

    expect(screen.getByTestId("tag-advanced")).toHaveTextContent("#advanced");
    expect(screen.getByTestId("tag-advanced")).toHaveTextContent("1");
  });

  it("displays recent notes sorted by update date", () => {
    render(() => <NotesSidebar notes={mockNotes} />);

    const recentLinks = screen.getAllByTestId(/^recent-/);
    expect(recentLinks[0]).toHaveAttribute("href", "/notes/note-1");
    expect(recentLinks[1]).toHaveAttribute("href", "/notes/note-2");
    expect(recentLinks[2]).toHaveAttribute("href", "/notes/note-3");
  });

  it("calls onFilterSelect when filter clicked", async () => {
    const onFilterSelect = vi.fn();
    render(() => <NotesSidebar notes={mockNotes} onFilterSelect={onFilterSelect} />);

    await fireEvent.click(screen.getByTestId("filter-linked"));
    expect(onFilterSelect).toHaveBeenCalledWith("linked");

    await fireEvent.click(screen.getByTestId("filter-orphaned"));
    expect(onFilterSelect).toHaveBeenCalledWith("orphaned");
  });

  it("calls onTagSelect when tag clicked", async () => {
    const onTagSelect = vi.fn();
    render(() => <NotesSidebar notes={mockNotes} onTagSelect={onTagSelect} />);

    await fireEvent.click(screen.getByTestId("tag-rust"));
    expect(onTagSelect).toHaveBeenCalledWith("rust");
  });

  it("toggles tag selection off when same tag clicked", async () => {
    const onTagSelect = vi.fn();
    render(() => <NotesSidebar notes={mockNotes} selectedTag="rust" onTagSelect={onTagSelect} />);

    await fireEvent.click(screen.getByTestId("tag-rust"));
    expect(onTagSelect).toHaveBeenCalledWith(undefined);
  });

  it("handles empty notes array", () => {
    render(() => <NotesSidebar notes={[]} />);
    expect(screen.getByTestId("notes-sidebar")).toBeInTheDocument();
    expect(screen.getByTestId("filter-all")).toHaveTextContent("All (0)");
  });
});
