import { useDensity } from "$lib/density-context";
import type { DensityMode } from "$lib/design-tokens";
import { Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

type CardProps = JSX.HTMLAttributes<HTMLDivElement> & { title?: string; density?: DensityMode };

export const Card: Component<CardProps> = (props) => {
  const [local, others] = splitProps(props, ["title", "density", "class", "children"]);
  const globalDensity = useDensity();
  const density = () => local.density || globalDensity;

  const paddingClass = () => {
    const mode = density();
    if (mode === "compact") return "p-3";
    if (mode === "spacious") return "p-8";
    return "p-6";
  };

  const headerPaddingClass = () => {
    const mode = density();
    if (mode === "compact") return "px-3 py-2";
    if (mode === "spacious") return "px-8 py-6";
    return "px-6 py-4";
  };

  return (
    <div class={`surface-01 border border-gray-800 rounded-lg overflow-hidden ${local.class || ""}`} {...others}>
      <Show when={local.title}>
        <div class={`${headerPaddingClass()} border-b border-gray-800`}>
          <h3 class="text-lg font-semibold text-white">{local.title}</h3>
        </div>
      </Show>
      <div class={`${paddingClass()} text-gray-300`}>{local.children}</div>
    </div>
  );
};
