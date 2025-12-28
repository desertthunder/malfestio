import { useNavigate } from "@solidjs/router";
import type { Component } from "solid-js";
import { DeckEditor } from "../components/DeckEditor";
import { api } from "../lib/api";
import type { Card, CreateDeckPayload } from "../lib/store";
import { toast } from "../lib/toast";

const DeckNew: Component = () => {
  const navigate = useNavigate();

  const handleSave = async (data: CreateDeckPayload) => {
    try {
      const { cards, ...deckPayload } = data;
      const res = await api.post("/decks", deckPayload);

      if (res.ok) {
        const deck = await res.json();

        if (cards && cards.length > 0) {
          await Promise.all(
            cards.map((c: Card) =>
              api.post("/cards", { deck_id: deck.id, front: c.front, back: c.back, media_url: c.mediaUrl })
            ),
          );
        }

        toast.success("Deck created successfully");
        navigate(`/decks/${deck.id}`);
      } else {
        const err = await res.json();
        toast.error(err.error || "Failed to create deck");
      }
    } catch (e) {
      console.error(e);
      toast.error("Network error");
    }
  };

  return (
    <div class="max-w-3xl mx-auto">
      <div class="mb-8">
        <h1 class="text-3xl font-light text-[#F4F4F4] mb-2 tracking-tight">Create New Deck</h1>
        <p class="text-[#C6C6C6] font-light">Start a new collection of flashcards.</p>
      </div>
      <DeckEditor onSave={handleSave} />
    </div>
  );
};

export default DeckNew;
