import { SearchInput } from "$components/SearchInput";
import { Skeleton } from "$components/ui/Skeleton";
import { Tag } from "$components/ui/Tag";
import { UserProfileCard } from "$components/UserProfileCard";
import { api } from "$lib/api";
import { authStore } from "$lib/store";
import { A } from "@solidjs/router";
import type { Component } from "solid-js";
import { createResource, For, Index, Show } from "solid-js";
import { Motion } from "solid-motionone";

const Discovery: Component = () => {
  const [data] = createResource(async () => {
    const res = await api.getDiscovery();
    if (res.ok) return (await res.json()) as { top_tags: [string, number][] };
    return { top_tags: [] };
  });

  const [profile] = createResource(() => authStore.user()?.did, async (did) => {
    if (!did) return null;
    const res = await api.getUserProfile(did);
    if (res.ok) return (await res.json());
    return null;
  });

  return (
    <Motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3 }}
      class="max-w-4xl mx-auto px-4 py-8 space-y-8">
      <Motion.div
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.4 }}
        class="text-center space-y-4">
        <h1 class="text-5xl text-[#F4F4F4] tracking-tight">Discover Malfestio</h1>
        <p class="text-xl text-[#C6C6C6] font-light">Explore community decks and popular topics</p>
        <div class="max-w-2xl mx-auto pt-4">
          <SearchInput />
        </div>
      </Motion.div>

      <Show when={authStore.isAuthenticated() && profile()}>
        <Motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4, delay: 0.1 }}>
          <UserProfileCard profile={profile()} />
        </Motion.div>
      </Show>

      <Motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.2 }}
        class="space-y-4">
        <h2 class="text-2xl text-[#F4F4F4] flex items-center gap-2">
          <span class="i-bi-tags-fill text-[#A855F7]" />
          Top Tags
        </h2>

        <Show
          when={!data.loading}
          fallback={
            <div class="flex gap-3 flex-wrap">
              <Index each={Array(8)}>{() => <Skeleton width="5rem" height="2.25rem" rounded="full" />}</Index>
            </div>
          }>
          <div class="flex flex-wrap gap-3">
            <For each={data()?.top_tags}>
              {(tag, i) => (
                <Motion.div
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ duration: 0.3, delay: i() * 0.03 }}>
                  <A
                    href={`/search?q=${encodeURIComponent(tag[0])}`}
                    class="group inline-flex items-center gap-2 px-4 py-2 bg-[#262626] border border-[#393939] rounded-full hover:border-[#0F62FE] transition-colors">
                    <Tag label={`#${tag[0]}`} color="purple" class="border-none bg-transparent px-0" />
                    <span class="text-xs text-[#8D8D8D] bg-[#161616] px-1.5 py-0.5 rounded-full">{tag[1]}</span>
                  </A>
                </Motion.div>
              )}
            </For>
            <Show when={data()?.top_tags.length === 0}>
              <p class="text-[#8D8D8D] font-light">No tags found yet. Create some decks!</p>
            </Show>
          </div>
        </Show>
      </Motion.div>
    </Motion.div>
  );
};

export default Discovery;
