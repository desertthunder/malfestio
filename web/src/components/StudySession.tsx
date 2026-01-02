import { scaleIn, slideInUp } from "$lib/animations";
import { api } from "$lib/api";
import type { Grade, ReviewCard } from "$lib/model";
import { Button } from "$ui/Button";
import { Dialog } from "$ui/Dialog";
import { ProgressBar } from "$ui/ProgressBar";
import { type Component, createEffect, createSignal, For, onCleanup, onMount, Show } from "solid-js";
import { Motion } from "solid-motionone";

type StudySessionProps = { cards: ReviewCard[]; onComplete: () => void; onExit: () => void };

const GRADE_LABELS: { [key in Grade]: { label: string; color: string; key: string } } = {
  0: { label: "Again", color: "text-red-500", key: "1" },
  1: { label: "Hard", color: "text-orange-500", key: "2" },
  2: { label: "Okay", color: "text-yellow-500", key: "3" },
  3: { label: "Good", color: "text-green-500", key: "4" },
  4: { label: "Easy", color: "text-emerald-500", key: "5" },
  5: { label: "Perfect", color: "text-cyan-500", key: "6" },
};

export const StudySession: Component<StudySessionProps> = (props) => {
  const [currentIndex, setCurrentIndex] = createSignal(0);
  const [isFlipped, setIsFlipped] = createSignal(false);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [showEditDialog, setShowEditDialog] = createSignal(false);

  const currentCard = () => props.cards[currentIndex()];
  const progress = () => ((currentIndex() + 1) / props.cards.length) * 100;
  const isComplete = () => currentIndex() >= props.cards.length;
  const handleFlip = () => setIsFlipped((f) => !f);

  const handleGrade = async (grade: Grade) => {
    const card = currentCard();
    if (!card || isSubmitting()) return;

    setIsSubmitting(true);
    try {
      const response = await api.submitReview(card.card_id, grade);
      if (response.ok) {
        await response.json();
        setIsFlipped(false);
        setCurrentIndex((i) => i + 1);
      }
    } catch (err) {
      console.error("Failed to submit review:", err);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (showEditDialog()) return;

    switch (e.key) {
      case " ":
        e.preventDefault();
        handleFlip();
        break;
      case "1":
        if (isFlipped()) handleGrade(0);
        break;
      case "2":
        if (isFlipped()) handleGrade(1);
        break;
      case "3":
        if (isFlipped()) handleGrade(2);
        break;
      case "4":
        if (isFlipped()) handleGrade(3);
        break;
      case "5":
        if (isFlipped()) handleGrade(4);
        break;
      case "6":
        if (isFlipped()) handleGrade(5);
        break;
      case "e":
      case "E":
        setShowEditDialog(true);
        break;
      case "Escape":
        props.onExit();
        break;
    }
  };

  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
  });

  onCleanup(() => {
    window.removeEventListener("keydown", handleKeyDown);
  });

  createEffect(() => {
    if (isComplete()) {
      props.onComplete();
    }
  });

  return (
    <div class="fixed inset-0 z-50 h-screen w-screen bg-gray-950 grid grid-rows-[auto_1fr_160px] overflow-hidden">
      <div class="w-full max-w-2xl mx-auto p-4 flex flex-col justify-end">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-400 text-sm">Card {currentIndex() + 1} of {props.cards.length}</span>
          <button onClick={() => props.onExit()} class="text-gray-400 hover:text-white text-sm flex items-center gap-1">
            ✕ Exit <span class="text-xs text-gray-500">(Esc)</span>
          </button>
        </div>
        <ProgressBar value={progress()} color="green" size="md" />
      </div>

      <div class="flex items-center justify-center p-4">
        <Show when={currentCard()} keyed>
          {(card) => (
            <Motion.div {...scaleIn} class="w-full max-w-2xl h-[400px]">
              <div
                onClick={handleFlip}
                class="relative w-full h-full cursor-pointer"
                style={{ "perspective": "1000px" }}>
                <div
                  class="relative w-full h-full transition-transform duration-500"
                  style={{
                    "transform-style": "preserve-3d",
                    "transform": isFlipped() ? "rotateY(180deg)" : "rotateY(0deg)",
                  }}>
                  <div
                    class="absolute inset-0 rounded-2xl bg-linear-to-br from-gray-800 to-gray-900 border border-gray-700 p-8 flex flex-col items-center justify-center"
                    style={{ "backface-visibility": "hidden" }}>
                    <span class="text-xs text-gray-500 mb-4">{card.deck_title}</span>
                    <p class="text-2xl text-white text-center font-medium">{card.front}</p>
                    <p class="text-gray-500 mt-8 text-sm animate-pulse">Press Space or click to reveal</p>
                  </div>

                  <div
                    class="absolute inset-0 rounded-2xl bg-linear-to-br from-gray-800 to-gray-900 border border-gray-700 p-8 flex flex-col items-center justify-center"
                    style={{ "backface-visibility": "hidden", "transform": "rotateY(180deg)" }}>
                    <span class="text-xs text-gray-500 mb-4">Answer</span>
                    <p class="text-2xl text-white text-center font-medium">{card.back}</p>
                    <Show when={card.hints.length > 0}>
                      <div class="mt-4 text-sm text-gray-400">
                        <For each={card.hints}>{(hint) => <p class="italic">💡 {hint}</p>}</For>
                      </div>
                    </Show>
                  </div>
                </div>
              </div>
            </Motion.div>
          )}
        </Show>
      </div>

      <div class="flex items-start justify-center p-4">
        <Show when={isFlipped()}>
          <Motion.div {...slideInUp} class="w-full max-w-2xl">
            <p class="text-center text-gray-400 text-sm mb-4">How well did you know this?</p>
            <div class="grid grid-cols-6 gap-2">
              <For each={[0, 1, 2, 3, 4, 5] as Grade[]}>
                {(grade) => (
                  <button
                    onClick={() => handleGrade(grade)}
                    disabled={isSubmitting()}
                    class="py-3 px-2 rounded-lg font-medium transition-colors bg-gray-800 hover:bg-gray-700 disabled:opacity-50 border border-transparent hover:border-gray-600 group">
                    <span
                      class={`block text-lg transition-transform group-hover:scale-110 ${GRADE_LABELS[grade].color}`}>
                      {GRADE_LABELS[grade].label}
                    </span>
                    <span class="block text-xs opacity-75 text-gray-400">({GRADE_LABELS[grade].key})</span>
                  </button>
                )}
              </For>
            </div>
          </Motion.div>
        </Show>
      </div>

      <div class="fixed bottom-4 left-1/2 -translate-x-1/2 text-gray-600 text-xs flex gap-4">
        <span>Space: Flip</span>
        <Show when={isFlipped()}>
          <span>1-6: Grade</span>
          <span>E: Edit</span>
        </Show>
        <span>Esc: Exit</span>
      </div>

      <Dialog open={showEditDialog()} onClose={() => setShowEditDialog(false)} title="Edit Card">
        <Show when={currentCard()}>
          {(card) => (
            <div class="space-y-4">
              <div>
                <label class="block text-sm text-gray-400 mb-1">Front</label>
                <p class="text-white bg-gray-800 p-3 rounded">{card().front}</p>
              </div>
              <div>
                <label class="block text-sm text-gray-400 mb-1">Back</label>
                <p class="text-white bg-gray-800 p-3 rounded">{card().back}</p>
              </div>
              <p class="text-gray-500 text-sm">Full editing coming soon.</p>
              <Button onClick={() => setShowEditDialog(false)} variant="secondary" class="w-full">Close</Button>
            </div>
          )}
        </Show>
      </Dialog>
    </div>
  );
};
