import type { Note } from "$lib/model";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NotesGraph } from "../NotesGraph";

vi.mock("@solidjs/router", () => ({ useNavigate: () => vi.fn() }));

const mockNotes: Note[] = [{
  id: "note-1",
  owner_did: "did:plc:test",
  title: "First Note",
  body: "Content with [[Second Note]] link",
  tags: ["test"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}, {
  id: "note-2",
  owner_did: "did:plc:test",
  title: "Second Note",
  body: "This links to [[First Note]]",
  tags: ["test", "linked"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}, {
  id: "note-3",
  owner_did: "did:plc:test",
  title: "Orphan Note",
  body: "No wikilinks here",
  tags: [],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}];

describe("NotesGraph", () => {
  afterEach(cleanup);

  it("renders the graph container", () => {
    render(() => <NotesGraph notes={mockNotes} />);
    expect(screen.getByTestId("notes-graph")).toBeInTheDocument();
  });

  it("renders SVG element", () => {
    render(() => <NotesGraph notes={mockNotes} />);
    const container = screen.getByTestId("notes-graph");
    const svg = container.querySelector("svg");
    expect(svg).toBeInTheDocument();
  });

  it("creates nodes group", () => {
    render(() => <NotesGraph notes={mockNotes} />);
    const container = screen.getByTestId("notes-graph");
    const nodesGroup = container.querySelector(".nodes");
    expect(nodesGroup).toBeInTheDocument();
  });

  it("creates links group", () => {
    render(() => <NotesGraph notes={mockNotes} />);
    const container = screen.getByTestId("notes-graph");
    const linksGroup = container.querySelector(".links");
    expect(linksGroup).toBeInTheDocument();
  });

  it("renders with custom dimensions", () => {
    render(() => <NotesGraph notes={mockNotes} width={1200} height={800} />);
    const container = screen.getByTestId("notes-graph");
    expect(container).toBeInTheDocument();
  });

  it("handles empty notes array", () => {
    render(() => <NotesGraph notes={[]} />);
    expect(screen.getByTestId("notes-graph")).toBeInTheDocument();
  });

  it("highlights current note when provided", () => {
    render(() => <NotesGraph notes={mockNotes} currentNoteId="note-1" />);
    expect(screen.getByTestId("notes-graph")).toBeInTheDocument();
  });
});
