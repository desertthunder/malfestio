import { Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

interface CardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  title?: string;
}

export const Card: Component<CardProps> = (props) => {
  const [local, others] = splitProps(props, ["title", "class", "children"]);

  return (
    <div class={`bg-gray-900 border border-gray-800 rounded-lg overflow-hidden ${local.class || ""}`} {...others}>
      <Show when={local.title}>
        <div class="px-6 py-4 border-b border-gray-800">
          <h3 class="text-lg font-semibold text-white">{local.title}</h3>
        </div>
      </Show>
      <div class="p-6 text-gray-300">{local.children}</div>
    </div>
  );
};
