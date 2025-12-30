import { useNavigate } from "@solidjs/router";
import clsx from "clsx";
import type { Component } from "solid-js";
import { createSignal } from "solid-js";

interface SearchInputProps {
  class?: string;
  initialQuery?: string;
}

export const SearchInput: Component<SearchInputProps> = (props) => {
  const [query, setQuery] = createSignal(props.initialQuery || "");
  const navigate = useNavigate();

  const handleSearch = (e: Event) => {
    e.preventDefault();
    if (query().trim()) {
      navigate(`/search?q=${encodeURIComponent(query())}`);
    }
  };

  return (
    <form onSubmit={handleSearch} class={clsx("relative", props.class)}>
      <div class="relative">
        <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
          <div class="i-bi-search text-gray-400" />
        </div>
        <input
          type="search"
          class="block w-full p-2 pl-10 text-sm border border-gray-300 rounded-lg bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
          placeholder="Search decks, cards..."
          value={query()}
          onInput={(e) => setQuery(e.currentTarget.value)} />
      </div>
    </form>
  );
};
