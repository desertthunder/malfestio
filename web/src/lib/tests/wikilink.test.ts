import { describe, expect, it } from "vitest";
import type { Note } from "../model";
import {
  extractHeadings,
  extractWikilinkTitles,
  findBacklinks,
  parseWikilinks,
  renderWikilinks,
  resolveWikilink,
  slugify,
} from "../wikilink";

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
  body: "This links to [[First Note]] and [[Third Note]]",
  tags: ["test"],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}, {
  id: "note-3",
  owner_did: "did:plc:test",
  title: "Third Note",
  body: "No wikilinks here",
  tags: [],
  visibility: { type: "Private" },
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
}];

describe("parseWikilinks", () => {
  it("extracts simple wikilinks", () => {
    const text = "Check out [[My Note]] for more info";
    const links = parseWikilinks(text);
    expect(links).toHaveLength(1);
    expect(links[0].title).toBe("My Note");
    expect(links[0].alias).toBeUndefined();
  });

  it("extracts wikilinks with aliases", () => {
    const text = "See [[Long Note Title|short alias]] here";
    const links = parseWikilinks(text);
    expect(links).toHaveLength(1);
    expect(links[0].title).toBe("Long Note Title");
    expect(links[0].alias).toBe("short alias");
  });

  it("extracts multiple wikilinks", () => {
    const text = "Both [[Note A]] and [[Note B|B]] are relevant";
    const links = parseWikilinks(text);
    expect(links).toHaveLength(2);
    expect(links[0].title).toBe("Note A");
    expect(links[1].title).toBe("Note B");
    expect(links[1].alias).toBe("B");
  });

  it("returns empty array for no wikilinks", () => {
    const text = "Just regular text without links";
    const links = parseWikilinks(text);
    expect(links).toHaveLength(0);
  });

  it("includes position information", () => {
    const text = "Start [[Link]] end";
    const links = parseWikilinks(text);
    expect(links[0].start).toBe(6);
    expect(links[0].end).toBe(14);
    expect(links[0].raw).toBe("[[Link]]");
  });
});

describe("extractWikilinkTitles", () => {
  it("returns unique titles", () => {
    const text = "[[Note A]] and [[Note B]] and [[Note A]] again";
    const titles = extractWikilinkTitles(text);
    expect(titles).toHaveLength(2);
    expect(titles).toContain("Note A");
    expect(titles).toContain("Note B");
  });
});

describe("resolveWikilink", () => {
  it("resolves exact match", () => {
    const note = resolveWikilink("First Note", mockNotes);
    expect(note?.id).toBe("note-1");
  });

  it("resolves case-insensitively", () => {
    const note = resolveWikilink("FIRST NOTE", mockNotes);
    expect(note?.id).toBe("note-1");
  });

  it("returns null for unresolved links", () => {
    const note = resolveWikilink("Nonexistent Note", mockNotes);
    expect(note).toBeNull();
  });

  it("trims whitespace", () => {
    const note = resolveWikilink("  First Note  ", mockNotes);
    expect(note?.id).toBe("note-1");
  });
});

describe("findBacklinks", () => {
  it("finds notes linking to target", () => {
    const backlinks = findBacklinks("Second Note", mockNotes);
    expect(backlinks).toHaveLength(1);
    expect(backlinks[0].id).toBe("note-1");
  });

  it("finds multiple backlinks", () => {
    const backlinks = findBacklinks("First Note", mockNotes);
    expect(backlinks).toHaveLength(1);
    expect(backlinks[0].id).toBe("note-2");
  });

  it("returns empty for notes with no backlinks", () => {
    const backlinks = findBacklinks("Third Note", mockNotes);
    expect(backlinks).toHaveLength(1);
    expect(backlinks[0].id).toBe("note-2");
  });

  it("is case-insensitive", () => {
    const backlinks = findBacklinks("SECOND NOTE", mockNotes);
    expect(backlinks).toHaveLength(1);
  });
});

describe("slugify", () => {
  it("converts to lowercase", () => {
    expect(slugify("Hello World")).toBe("hello-world");
  });

  it("removes special characters", () => {
    expect(slugify("Hello, World!")).toBe("hello-world");
  });

  it("collapses multiple dashes", () => {
    expect(slugify("Hello   World")).toBe("hello-world");
  });

  it("handles empty string", () => {
    expect(slugify("")).toBe("");
  });
});

describe("extractHeadings", () => {
  it("extracts H1-H6 headings", () => {
    const markdown = `# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6`;
    const headings = extractHeadings(markdown);
    expect(headings).toHaveLength(6);
    expect(headings[0]).toEqual({ level: 1, text: "Heading 1", id: "heading-1" });
    expect(headings[5]).toEqual({ level: 6, text: "Heading 6", id: "heading-6" });
  });

  it("handles special characters in headings", () => {
    const markdown = "## Hello, World!";
    const headings = extractHeadings(markdown);
    expect(headings[0].id).toBe("hello-world");
  });

  it("returns empty array for no headings", () => {
    const markdown = "Just regular text";
    expect(extractHeadings(markdown)).toHaveLength(0);
  });
});

describe("renderWikilinks", () => {
  it("renders resolved links as anchors", () => {
    const text = "See [[My Note]] here";
    const result = renderWikilinks(text, () => "/notes/123");
    expect(result).toBe("See <a href=\"/notes/123\" class=\"wikilink\">My Note</a> here");
  });

  it("renders unresolved links as spans", () => {
    const text = "See [[Missing]] here";
    const result = renderWikilinks(text, () => null);
    expect(result).toBe("See <span class=\"wikilink wikilink-broken\">Missing</span> here");
  });

  it("uses alias for display text when provided", () => {
    const text = "See [[Long Title|alias]] here";
    const result = renderWikilinks(text, () => "/notes/123");
    expect(result).toBe("See <a href=\"/notes/123\" class=\"wikilink\">alias</a> here");
  });
});
