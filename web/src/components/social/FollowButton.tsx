import { api } from "$lib/api";
import { authStore } from "$lib/store";
import { Button } from "$ui/Button";
import { createSignal, onMount, Show } from "solid-js";

type FollowButtonProps = { did: string; initialIsFollowing?: boolean };

export function FollowButton(props: FollowButtonProps) {
  const [isFollowing, setIsFollowing] = createSignal(props.initialIsFollowing || false);
  const [loading, setLoading] = createSignal(false);

  onMount(async () => {
    if (props.initialIsFollowing === undefined) {
      const user = authStore.user();
      if (user) {
        try {
          const res = await api.getFollowers(props.did);
          if (res.ok) {
            const followers: string[] = await res.json();
            setIsFollowing(followers.includes(user.did));
          }
        } catch (e) {
          console.error("Failed to check follow status", e);
        }
      }
    }
  });

  const toggle = async () => {
    setLoading(true);
    try {
      if (isFollowing()) {
        await api.unfollow(props.did);
        setIsFollowing(false);
      } else {
        await api.follow(props.did);
        setIsFollowing(true);
      }
    } catch (e) {
      console.error("Failed to toggle follow", e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button onClick={toggle} disabled={loading()} variant={isFollowing() ? "secondary" : undefined}>
      <Show when={isFollowing()} fallback="Follow">Unfollow</Show>
    </Button>
  );
}
