import { useDensity } from "$lib/density-context";
import type { DensityMode } from "$lib/design-tokens";
import { Show, splitProps } from "solid-js";
import type { Component } from "solid-js";

export type TagType = "read-only" | "dismissible" | "selectable";
export type TagColor = "gray" | "blue" | "green" | "red" | "yellow" | "purple";

type TagProps = {
  label: string;
  type?: TagType;
  color?: TagColor;
  selected?: boolean;
  onDismiss?: () => void;
  onSelect?: () => void;
  icon?: Component;
  density?: DensityMode;
  class?: string;
};

const colorStyles: Record<TagColor, { base: string; selected: string }> = {
  gray: { base: "bg-gray-800 text-gray-300 border-gray-700", selected: "bg-gray-600 text-white border-gray-500" },
  blue: { base: "bg-blue-900/40 text-blue-300 border-blue-800", selected: "bg-blue-600 text-white border-blue-500" },
  green: {
    base: "bg-green-900/40 text-green-300 border-green-800",
    selected: "bg-green-600 text-white border-green-500",
  },
  red: { base: "bg-red-900/40 text-red-300 border-red-800", selected: "bg-red-600 text-white border-red-500" },
  yellow: {
    base: "bg-yellow-900/40 text-yellow-300 border-yellow-800",
    selected: "bg-yellow-600 text-white border-yellow-500",
  },
  purple: {
    base: "bg-purple-900/40 text-purple-300 border-purple-800",
    selected: "bg-purple-600 text-white border-purple-500",
  },
};

export const Tag: Component<TagProps> = (props) => {
  const [local, _others] = splitProps(props, [
    "label",
    "type",
    "color",
    "selected",
    "onDismiss",
    "onSelect",
    "icon",
    "density",
    "class",
  ]);

  const globalDensity = useDensity();
  const density = () => local.density || globalDensity;
  const type = () => local.type ?? "read-only";
  const color = () => local.color ?? "gray";

  const baseClass = () => {
    const c = colorStyles[color()];
    const isSelected = local.selected && type() === "selectable";
    return isSelected ? c.selected : c.base;
  };

  const sizeClass = () => {
    const d = density();
    return d === "compact" ? "px-2 py-0.5 text-xs" : d === "spacious" ? "px-3 py-1.5 text-sm" : "px-2.5 py-1 text-xs";
  };

  const handleClick = () => {
    if (type() === "selectable") {
      local.onSelect?.();
    }
  };

  const handleDismiss = (e: MouseEvent) => {
    e.stopPropagation();
    local.onDismiss?.();
  };

  return (
    <span
      onClick={handleClick}
      role={type() === "selectable" ? "button" : undefined}
      tabIndex={type() === "selectable" ? 0 : undefined}
      aria-pressed={type() === "selectable" ? local.selected : undefined}
      class={`
        inline-flex items-center gap-1.5 font-medium border rounded-full transition-colors
        ${sizeClass()}
        ${baseClass()}
        ${type() === "selectable" ? "cursor-pointer hover:opacity-80" : ""}
        ${local.class || ""}
      `}>
      <Show when={local.icon}>
        {icon => {
          const IconComponent = icon();
          return (
            <span class="w-3 h-3 flex items-center justify-center">
              <IconComponent />
            </span>
          );
        }}
      </Show>
      <span>{local.label}</span>
      <Show when={type() === "dismissible"}>
        <button
          type="button"
          onClick={handleDismiss}
          class="w-4 h-4 flex items-center justify-center rounded-full hover:bg-white/10 transition-colors"
          aria-label={`Remove ${local.label}`}>
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </Show>
    </span>
  );
};
