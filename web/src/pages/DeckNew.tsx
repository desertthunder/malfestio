import { DeckEditor } from "$components/DeckEditor";
import { TutorialOverlay } from "$components/TutorialOverlay";
import { api } from "$lib/api";
import type { CreateDeckPayload } from "$lib/model";
import { authStore, prefStore } from "$lib/store";
import { syncStore } from "$lib/sync-store";
import { toast } from "$lib/toast";
import { TutorialProvider, useTutorial } from "$lib/TutorialProvider";
import { Button } from "$ui/Button";
import { useNavigate } from "@solidjs/router";
import type { Component } from "solid-js";
import { createEffect, onMount, Show } from "solid-js";
import { Motion } from "solid-motionone";

const DeckNewContent: Component = () => {
  const navigate = useNavigate();
  const tutorial = useTutorial();

  onMount(async () => {
    if (!prefStore.prefs()) {
      await prefStore.fetchPrefs();
    }
  });

  createEffect(() => {
    if (tutorial.shouldShowTutorial() && !tutorial.active()) {
      setTimeout(() => tutorial.startTutorial(), 500);
    }
  });

  const handleSave = async (data: CreateDeckPayload) => {
    try {
      const user = authStore.user();
      if (!user) {
        toast.error("Not authenticated");
        return;
      }

      const localDeck = await syncStore.saveDeckLocally({
        ownerDid: user.did,
        title: data.title,
        description: data.description ?? "",
        tags: data.tags ?? [],
        visibility: data.visibility ?? { type: "Private" },
      });

      for (const card of data.cards ?? []) {
        await syncStore.saveCardLocally({
          deckId: localDeck.id,
          front: card.front,
          back: card.back,
          cardType: card.cardType ?? "basic",
          hints: card.hints ?? [],
        });
      }

      if (syncStore.isOnline()) {
        const res = await api.createDeck(data);
        if (res.ok) {
          const serverDeck = await res.json();
          if (!prefStore.prefs()?.tutorial_deck_completed) {
            await api.updatePreferences({ tutorial_deck_completed: true });
            prefStore.fetchPrefs();
          }
          toast.success("Deck created and synced");
          navigate(`/decks/${serverDeck.id}`);
          return;
        }
      }

      if (!prefStore.prefs()?.tutorial_deck_completed) {
        await api.updatePreferences({ tutorial_deck_completed: true }).catch(() => {});
        prefStore.fetchPrefs();
      }
      toast.success("Deck saved locally");
      navigate(`/decks/${localDeck.id}`);
    } catch (e) {
      console.error(e);
      toast.error("Failed to save deck");
    }
  };

  return (
    <>
      <Motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.3 }}
        class="max-w-3xl mx-auto">
        <Motion.div
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
          class="mb-8 flex justify-between items-start">
          <div>
            <h1 class="text-4xl text-[#F4F4F4] mb-2 tracking-tight">Create New Deck</h1>
            <p class="text-[#C6C6C6] font-light">Start a new collection of flashcards.</p>
          </div>
          <Show when={prefStore.prefs()?.tutorial_deck_completed}>
            <Button variant="secondary" size="sm" onClick={() => tutorial.startTutorial()}>
              <span class="i-bi-question-circle mr-1.5" /> Show Tutorial
            </Button>
          </Show>
        </Motion.div>
        <Motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: 0.1 }}>
          <DeckEditor onSave={handleSave} />
        </Motion.div>
      </Motion.div>

      <TutorialOverlay />
    </>
  );
};

const DeckNew: Component = () => (
  <TutorialProvider>
    <DeckNewContent />
  </TutorialProvider>
);

export default DeckNew;
