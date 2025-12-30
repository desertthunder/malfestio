import { ReviewStats } from "$components/ReviewStats";
import { StudySession } from "$components/StudySession";
import { fadeIn } from "$lib/animations";
import { api } from "$lib/api";
import type { ReviewCard, StudyStats as StudyStatsType } from "$lib/model";
import { Button } from "$ui/Button";
import { Skeleton } from "$ui/Skeleton";
import { useNavigate, useParams } from "@solidjs/router";
import { type Component, createSignal, onMount, Show } from "solid-js";
import { Motion } from "solid-motionone";

const Review: Component = () => {
  const params = useParams<{ deckId?: string }>();
  const navigate = useNavigate();

  const [cards, setCards] = createSignal<ReviewCard[]>([]);
  const [stats, setStats] = createSignal<StudyStatsType | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [sessionActive, setSessionActive] = createSignal(false);
  const [sessionComplete, setSessionComplete] = createSignal(false);

  onMount(async () => {
    const [statsRes, cardsRes] = await Promise.all([api.getStats(), api.getDueCards(params.deckId)]);
    if (statsRes.ok) setStats(await statsRes.json());
    if (cardsRes.ok) setCards(await cardsRes.json());
    setLoading(false);
  });

  const startSession = () => {
    if (cards().length > 0) {
      setSessionActive(true);
      setSessionComplete(false);
    }
  };

  const handleComplete = async () => {
    setSessionActive(false);
    setSessionComplete(true);
    const res = await api.getStats();
    if (res.ok) setStats(await res.json());
  };

  const handleExit = () => {
    setSessionActive(false);
    navigate("/");
  };

  return (
    <Show
      when={!sessionActive()}
      fallback={<StudySession cards={cards()} onComplete={handleComplete} onExit={handleExit} />}>
      <Motion.div {...fadeIn} class="max-w-4xl mx-auto py-8 px-4">
        <h1 class="text-3xl font-bold text-white mb-8">{params.deckId ? "Deck Review" : "Daily Review"}</h1>

        <ReviewStats stats={stats()} loading={loading()} />

        <div class="mt-8 bg-gray-900 rounded-xl p-6 border border-gray-800">
          <Show
            when={!loading()}
            fallback={
              <div class="space-y-4">
                <Skeleton class="h-8 w-48" />
                <Skeleton class="h-12 w-full" />
              </div>
            }>
            <Show
              when={cards().length > 0}
              fallback={
                <div class="text-center py-8">
                  {/* TODO: replace with an icon */}
                  <p class="text-4xl mb-4">🎉</p>
                  <Show
                    when={sessionComplete()}
                    fallback={
                      <>
                        <h2 class="text-xl font-semibold text-white mb-2">All Caught Up!</h2>
                        <p class="text-gray-400 mb-6">You have no cards due for review right now.</p>
                      </>
                    }>
                    <>
                      <h2 class="text-xl font-semibold text-white mb-2">Session Complete!</h2>
                      <p class="text-gray-400 mb-6">Great job! You've reviewed all your due cards.</p>
                    </>
                  </Show>
                  <Button onClick={() => navigate("/")} variant="secondary">Back to Library</Button>
                </div>
              }>
              <div class="text-center py-4">
                <p class="text-lg text-white mb-2">
                  You have <span class="font-bold text-blue-400">{cards().length}</span> cards due
                </p>
                <p class="text-gray-400 text-sm mb-6">Use keyboard shortcuts for faster reviews</p>
                <Button onClick={startSession} class="px-8 py-3 text-lg">Start Study Session</Button>
              </div>
            </Show>
          </Show>
        </div>

        <Motion.div {...fadeIn} class="mt-8 bg-gray-900/50 rounded-xl p-6 border border-gray-800/50">
          <h3 class="text-sm font-semibold text-gray-400 mb-4">Keyboard Shortcuts</h3>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div class="flex items-center gap-2">
              <kbd class="px-2 py-1 bg-gray-800 rounded text-gray-300">Space</kbd>
              <span class="text-gray-400">Flip card</span>
            </div>
            <div class="flex items-center gap-2">
              <kbd class="px-2 py-1 bg-gray-800 rounded text-gray-300">1-5</kbd>
              <span class="text-gray-400">Grade answer</span>
            </div>
            <div class="flex items-center gap-2">
              <kbd class="px-2 py-1 bg-gray-800 rounded text-gray-300">E</kbd>
              <span class="text-gray-400">Edit card</span>
            </div>
            <div class="flex items-center gap-2">
              <kbd class="px-2 py-1 bg-gray-800 rounded text-gray-300">Esc</kbd>
              <span class="text-gray-400">Exit session</span>
            </div>
          </div>
        </Motion.div>
      </Motion.div>
    </Show>
  );
};

export default Review;
