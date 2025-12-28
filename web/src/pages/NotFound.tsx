import { A } from "@solidjs/router";
import type { Component } from "solid-js";

const NotFound: Component = () => {
  return (
    <div class="max-w-7xl mx-auto px-6 py-24 text-center">
      <h1 class="text-8xl font-thin text-[#393939] mb-4">404</h1>
      <h2 class="text-2xl font-light text-[#F4F4F4] mb-6">Page not found</h2>
      <p class="text-[#C6C6C6] mb-12 max-w-md mx-auto font-light">
        The page you are looking for might have been removed, had its name changed, or is temporarily unavailable.
      </p>
      <A
        href="/"
        class="bg-[#0F62FE] hover:bg-[#0353E9] text-white px-8 py-3 font-medium text-sm transition-colors inline-block">
        Go to Library
      </A>
    </div>
  );
};

export default NotFound;
