/**
 * Wikilink parsing, resolution, and heading extraction utilities
 * Supports [[Note Title]] syntax for linking between notes
 */

import type { Note } from "./model";

/** Regex pattern to match wikilinks: [[Title]] or [[Title|Alias]] */
const WIKILINK_PATTERN = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;

/** Regex pattern to match markdown headings (H1-H6) */
const HEADING_PATTERN = /^(#{1,6})\s+(.+)$/gm;

/**
 * Represents a parsed wikilink
 */
export type WikiLink = {
  /** The full match including brackets */
  raw: string;
  /** The title of the linked note */
  title: string;
  /** Optional display alias */
  alias?: string;
  /** Start index in the source text */
  start: number;
  /** End index in the source text */
  end: number;
};

/**
 * Represents a heading extracted from markdown
 */
export type Heading = {
  /** Heading level (1-6) */
  level: number;
  /** Heading text content */
  text: string;
  /** Slugified ID for anchor links */
  id: string;
};

/**
 * Parse all wikilinks from markdown text
 *
 * @param text - Markdown text containing wikilinks
 * @returns Array of parsed wikilinks with position info
 */
export function parseWikilinks(text: string): WikiLink[] {
  const links: WikiLink[] = [];
  let match: RegExpExecArray | null;

  // Reset regex state
  WIKILINK_PATTERN.lastIndex = 0;

  while ((match = WIKILINK_PATTERN.exec(text)) !== null) {
    links.push({
      raw: match[0],
      title: match[1].trim(),
      alias: match[2]?.trim(),
      start: match.index,
      end: match.index + match[0].length,
    });
  }

  return links;
}

/**
 * Extract unique wikilink titles from text
 *
 * @param text - Markdown text containing wikilinks
 * @returns Array of unique linked note titles
 */
export function extractWikilinkTitles(text: string): string[] {
  const links = parseWikilinks(text);
  const titles = new Set(links.map((l) => l.title));
  return Array.from(titles);
}

/**
 * Resolve a wikilink title to a note (case-insensitive)
 *
 * @param title - The wikilink title to resolve
 * @param notes - Array of notes to search
 * @returns The matching note or null if not found
 */
export function resolveWikilink(title: string, notes: Note[]): Note | null {
  const normalizedTitle = title.toLowerCase().trim();
  return notes.find((n) => n.title.toLowerCase().trim() === normalizedTitle) ?? null;
}

/**
 * Find all notes that contain wikilinks to a target note
 *
 * @param noteTitle - Title of the target note
 * @param allNotes - Array of all notes to search
 * @returns Array of notes that link to the target
 */
export function findBacklinks(noteTitle: string, allNotes: Note[]): Note[] {
  const normalizedTitle = noteTitle.toLowerCase().trim();
  return allNotes.filter((note) => {
    const links = extractWikilinkTitles(note.body);
    return links.some((link) => link.toLowerCase().trim() === normalizedTitle);
  });
}

/**
 * Convert heading text to a URL-safe slug
 *
 * @param text - Heading text to slugify
 * @returns Slugified string suitable for anchor IDs
 */
export function slugify(text: string): string {
  return text.toLowerCase().trim().replace(/[^\w\s-]/g, "").replace(/\s+/g, "-").replace(/-+/g, "-");
}

/**
 * Extract all headings from markdown text
 *
 * @param markdown - Markdown text to parse
 * @returns Array of headings with level, text, and id
 */
export function extractHeadings(markdown: string): Heading[] {
  const headings: Heading[] = [];
  let match: RegExpExecArray | null;

  HEADING_PATTERN.lastIndex = 0;

  while ((match = HEADING_PATTERN.exec(markdown)) !== null) {
    const level = match[1].length as 1 | 2 | 3 | 4 | 5 | 6;
    const text = match[2].trim();
    headings.push({ level, text, id: slugify(text) });
  }

  return headings;
}

/**
 * Render wikilinks as HTML anchor tags
 *
 * @param text - Text containing wikilinks
 * @param resolveHref - Function to resolve a title to an href
 * @returns Text with wikilinks replaced by anchor tags
 */
export function renderWikilinks(text: string, resolveHref: (title: string) => string | null): string {
  const href = resolveHref(text.trim());
  return text.replace(
    WIKILINK_PATTERN,
    (_, title: string, alias?: string) =>
      href
        ? `<a href="${href}" class="wikilink">${(alias || title).trim()}</a>`
        : `<span class="wikilink wikilink-broken">${(alias || title).trim()}</span>`,
  );
}
