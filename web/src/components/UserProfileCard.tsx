import { Card } from "$components/ui/Card";
import type { UserProfile } from "$lib/model";
import type { Component } from "solid-js";

type UserProfileCardProps = { profile: UserProfile; class?: string };

export const UserProfileCard: Component<UserProfileCardProps> = (props) => {
  return (
    <Card class={`p-6 ${props.class || ""}`}>
      <div class="flex items-center gap-4 mb-4">
        <div class="h-16 w-16 rounded-full bg-primary-100 text-primary-700 flex items-center justify-center text-2xl font-bold">
          {props.profile.did.slice(4, 6).toUpperCase()}
        </div>
        <div>
          <h3 class="text-xl font-bold truncate max-w-[200px]" title={props.profile.did}>{props.profile.did}</h3>
          <p class="text-gray-500 text-sm">AT Protocol User</p>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-4 text-center border-t border-gray-100 dark:border-gray-700 pt-4">
        <div>
          <div class="text-2xl font-bold text-gray-900 dark:text-white">{props.profile.follower_count}</div>
          <div class="text-xs text-gray-500 uppercase tracking-wide">Followers</div>
        </div>
        <div>
          <div class="text-2xl font-bold text-gray-900 dark:text-white">{props.profile.following_count}</div>
          <div class="text-xs text-gray-500 uppercase tracking-wide">Following</div>
        </div>
        <div>
          <div class="text-2xl font-bold text-gray-900 dark:text-white">
            {props.profile.deck_count + props.profile.indexed_deck_count}
          </div>
          <div class="text-xs text-gray-500 uppercase tracking-wide">Decks</div>
        </div>
      </div>
    </Card>
  );
};
