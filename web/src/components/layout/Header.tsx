import { A } from "@solidjs/router";
import { type Component, Show } from "solid-js";
import { authStore } from "../../lib/store";
import { Avatar } from "../ui/Avatar";

const Login: Component = () => (
  <A href="/login" class="px-4 py-2 bg-white text-gray-900 text-sm font-medium hover:bg-gray-100 transition-colors">
    Login
  </A>
);

export const Header: Component = () => {
  return (
    <header class="h-16 border-b border-gray-800 bg-gray-900 flex items-center justify-between px-6 sticky top-0 z-50">
      <div class="flex items-center gap-6">
        <A href="/" class="text-xl font-bold text-white tracking-tight">Malfestio</A>
        <nav class="hidden md:flex items-center gap-4 text-sm font-medium text-gray-400">
          <A href="/decks" activeClass="text-blue-500" class="hover:text-white transition-colors">Decks</A>
          <A href="/review" activeClass="text-blue-500" class="hover:text-white transition-colors">Review</A>
        </nav>
      </div>
      <div class="flex items-center gap-4">
        <Show when={authStore.user()} fallback={<Login />}>
          <div class="flex items-center gap-3">
            <span class="text-xs text-gray-400">{authStore.user()?.handle}</span>
            <button
              onClick={() => authStore.logout()}
              class="text-xs text-red-400 hover:text-red-300 transition-colors">
              Logout
            </button>
            <Avatar name={authStore.user()?.handle} size="sm" />
          </div>
        </Show>
      </div>
    </header>
  );
};
