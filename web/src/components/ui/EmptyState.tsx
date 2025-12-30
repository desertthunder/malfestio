import { Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

type EmptyStateProps = {
  title: string;
  description?: string;
  icon?: JSX.Element;
  action?: JSX.Element;
  class?: string;
};

const DefaultIcon: Component = () => (
  <svg class="w-12 h-12 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="1.5"
      d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
  </svg>
);

export const EmptyState: Component<EmptyStateProps> = (props) => {
  const [local, _others] = splitProps(props, ["title", "description", "icon", "action", "class"]);

  return (
    <div class={`flex flex-col items-center justify-center py-12 px-4 text-center ${local.class || ""}`}>
      <div class="mb-4">{local.icon ?? <DefaultIcon />}</div>
      <h3 class="text-lg font-semibold text-white mb-2">{local.title}</h3>
      <Show when={local.description}>
        <p class="text-sm text-gray-400 max-w-sm mb-6">{local.description}</p>
      </Show>
      <Show when={local.action}>{local.action}</Show>
    </div>
  );
};
