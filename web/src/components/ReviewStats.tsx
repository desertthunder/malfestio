import { fadeIn } from "$lib/animations";
import { api } from "$lib/api";
import type { ReviewCard, StudyStats } from "$lib/store";
import { Skeleton } from "$ui/Skeleton";
import type { Component } from "solid-js";
import { Motion } from "solid-motionone";

type ReviewStatsProps = { stats: StudyStats | null; loading?: boolean };

export const ReviewStats: Component<ReviewStatsProps> = (props) => {
  return (
    <Motion.div {...fadeIn} class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      {/* TODO: use solid conditional components instead of ternary */}
      {props.loading
        ? (
          <div class="space-y-4">
            <Skeleton class="h-6 w-32" />
            <Skeleton class="h-4 w-48" />
            <Skeleton class="h-4 w-40" />
          </div>
        )
        : props.stats
        ? (
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-lg font-semibold text-white">Study Progress</h3>
              {/* TODO: fire icon */}
              <span class="text-2xl">🔥 {props.stats.current_streak} day streak</span>
            </div>

            <div class="grid grid-cols-3 gap-4 text-center">
              <div class="bg-gray-800 rounded-lg p-4">
                <p class="text-3xl font-bold text-blue-400">{props.stats.due_count}</p>
                <p class="text-sm text-gray-400">Due Today</p>
              </div>
              <div class="bg-gray-800 rounded-lg p-4">
                <p class="text-3xl font-bold text-green-400">{props.stats.reviewed_today}</p>
                <p class="text-sm text-gray-400">Reviewed</p>
              </div>
              <div class="bg-gray-800 rounded-lg p-4">
                <p class="text-3xl font-bold text-purple-400">{props.stats.total_reviews}</p>
                <p class="text-sm text-gray-400">Total</p>
              </div>
            </div>

            {props.stats.longest_streak > 0 && (
              <p class="text-sm text-gray-500 text-center">Longest streak: {props.stats.longest_streak} days</p>
            )}
          </div>
        )
        : <p class="text-gray-400">No stats available</p>}
    </Motion.div>
  );
};

// TODO: move this to api.ts
export async function fetchStudyStats(): Promise<StudyStats | null> {
  try {
    const response = await api.getStats();
    if (response.ok) {
      return response.json();
    }
  } catch (err) {
    console.error("Failed to fetch stats:", err);
  }
  return null;
}

// TODO: move this to api.ts
export async function fetchDueCards(deckId?: string): Promise<ReviewCard[]> {
  try {
    const response = await api.getDueCards(deckId);
    if (response.ok) {
      return response.json();
    }
  } catch (err) {
    console.error("Failed to fetch due cards:", err);
  }
  return [];
}
