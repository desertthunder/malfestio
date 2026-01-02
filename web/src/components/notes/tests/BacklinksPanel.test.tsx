import type { Note } from "$lib/model";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { BacklinksPanel } from "../BacklinksPanel";

vi.mock(
  "@solidjs/router",
  () => ({ A: (props: { href: string; children: JSX.Element }) => <a href={props.href}>{props.children}</a> }),
);

describe("BacklinksPanel", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockBacklinks: Note[] = [{
    id: "note-1",
    owner_did: "did:plc:test",
    title: "First Backlink",
    body: "Content",
    tags: [],
    visibility: { type: "Private" },
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  }, {
    id: "note-2",
    owner_did: "did:plc:test",
    title: "Second Backlink",
    body: "Content",
    tags: [],
    visibility: { type: "Private" },
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  }];

  it("renders heading title", () => {
    render(() => <BacklinksPanel backlinks={[]} />);
    expect(screen.getByText("Backlinks")).toBeInTheDocument();
  });

  it("shows empty state when no backlinks", () => {
    render(() => <BacklinksPanel backlinks={[]} />);
    expect(screen.getByText("No incoming links")).toBeInTheDocument();
  });

  it("renders all backlinks", () => {
    render(() => <BacklinksPanel backlinks={mockBacklinks} />);
    expect(screen.getByText("First Backlink")).toBeInTheDocument();
    expect(screen.getByText("Second Backlink")).toBeInTheDocument();
  });

  it("displays backlink count", () => {
    render(() => <BacklinksPanel backlinks={mockBacklinks} />);
    expect(screen.getByText("(2)")).toBeInTheDocument();
  });

  it("renders backlinks as navigation links", () => {
    render(() => <BacklinksPanel backlinks={mockBacklinks} />);
    const firstLink = screen.getByRole("link", { name: /First Backlink/ });
    expect(firstLink).toHaveAttribute("href", "/notes/note-1");
  });
});
