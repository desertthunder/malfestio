import { fadeIn } from "$lib/animations";
import type { StudyStats } from "$lib/model";
import { Skeleton } from "$ui/Skeleton";
import { type Component, Show } from "solid-js";
import { Motion } from "solid-motionone";

type ReviewStatsProps = { stats: StudyStats | null; loading?: boolean };

export const ReviewStats: Component<ReviewStatsProps> = (props) => (
  <Motion.div {...fadeIn} class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <Show
      when={!props.loading}
      fallback={
        <div class="space-y-4">
          <Skeleton class="h-6 w-32" />
          <Skeleton class="h-4 w-48" />
          <Skeleton class="h-4 w-40" />
        </div>
      }>
      <Show when={props.stats} fallback={<p class="text-gray-400">No stats available</p>}>
        {stats => (
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-lg font-semibold text-white">Study Progress</h3>
              <span class="text-2xl flex items-center gap-2">
                <i class="i-bi-fire" />
                <span>{stats().current_streak} day streak</span>
              </span>
            </div>

            <div class="grid grid-cols-3 gap-4 text-center">
              <div class="bg-gray-800 rounded-lg p-4">
                <p class="text-3xl font-bold text-blue-400">{stats().due_count}</p>
                <p class="text-sm text-gray-400">Due Today</p>
              </div>
              <div class="bg-gray-800 rounded-lg p-4">
                <p class="text-3xl font-bold text-green-400">{stats().reviewed_today}</p>
                <p class="text-sm text-gray-400">Reviewed</p>
              </div>
              <div class="bg-gray-800 rounded-lg p-4">
                <p class="text-3xl font-bold text-purple-400">{stats().total_reviews}</p>
                <p class="text-sm text-gray-400">Total</p>
              </div>
            </div>
            <Show when={stats().longest_streak > 0}>
              <p class="text-sm text-gray-500 text-center">Longest streak: {stats().longest_streak} days</p>
            </Show>
          </div>
        )}
      </Show>
    </Show>
  </Motion.div>
);
