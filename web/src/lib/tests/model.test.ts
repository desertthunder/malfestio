import { describe, expect, it } from "vitest";
import { asCard, asDeck, asNote, type Persona, type SearchResult, type UserPreferences } from "../model";

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

describe("Persona Types", () => {
  it("accepts valid persona values", () => {
    const learner: Persona = "learner";
    const creator: Persona = "creator";
    const curator: Persona = "curator";

    expect(learner).toBe("learner");
    expect(creator).toBe("creator");
    expect(curator).toBe("curator");
  });
});

describe("UserPreferences Types", () => {
  it("accepts valid user preferences object", () => {
    const prefs: UserPreferences = {
      user_did: "did:plc:test",
      persona: "learner",
      onboarding_completed_at: "2024-01-01T00:00:00Z",
      tutorial_deck_completed: false,
    };

    expect(prefs.user_did).toBe("did:plc:test");
    expect(prefs.persona).toBe("learner");
    expect(prefs.onboarding_completed_at).toBe("2024-01-01T00:00:00Z");
    expect(prefs.tutorial_deck_completed).toBe(false);
  });

  it("accepts null values for optional fields", () => {
    const prefs: UserPreferences = {
      user_did: "did:plc:test",
      persona: null,
      onboarding_completed_at: null,
      tutorial_deck_completed: false,
    };

    expect(prefs.persona).toBeNull();
    expect(prefs.onboarding_completed_at).toBeNull();
  });
});
