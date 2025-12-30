import { describe, expect, it } from "vitest";
import { asCard, asDeck, asNote, type SearchResult } from "./model";

describe("Type Guards", () => {
  const deckResult: SearchResult = {
    item_type: "deck",
    item_id: "deck1",
    creator_did: "did:test",
    data: {
      id: "deck1",
      owner_did: "did:test",
      title: "Test Deck",
      description: "Description",
      tags: [],
      visibility: { type: "Public" },
    },
    rank: 1,
  };

  const cardResult: SearchResult = {
    item_type: "card",
    item_id: "card1",
    creator_did: "did:test",
    data: { front: "Front", back: "Back", deck_id: "deck1" },
    rank: 1,
  };

  const noteResult: SearchResult = {
    item_type: "note",
    item_id: "note1",
    creator_did: "did:test",
    data: { id: "note1", title: "Test Note", owner_did: "did:test" },
    rank: 1,
  };

  it("asDeck correctly identifies decks", () => {
    expect(asDeck(deckResult)).toBe(deckResult);
    expect(asDeck(cardResult)).toBeUndefined();
    expect(asDeck(noteResult)).toBeUndefined();
  });

  it("asCard correctly identifies cards", () => {
    expect(asCard(cardResult)).toBe(cardResult);
    expect(asCard(deckResult)).toBeUndefined();
    expect(asCard(noteResult)).toBeUndefined();
  });

  it("asNote correctly identifies notes", () => {
    expect(asNote(noteResult)).toBe(noteResult);
    expect(asNote(deckResult)).toBeUndefined();
    expect(asNote(cardResult)).toBeUndefined();
  });
});
