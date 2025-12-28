import { A } from "@solidjs/router";
import type { Component } from "solid-js";

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
        {/* Placeholder for Auth/User menu */}
        <div class="w-8 h-8 rounded-full bg-gray-700" />
      </div>
    </header>
  );
};
