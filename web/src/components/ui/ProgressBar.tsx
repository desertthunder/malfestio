import { Show, splitProps } from "solid-js";
import type { Component } from "solid-js";

type ProgressBarSize = "sm" | "md" | "lg";
type ProgressBarColor = "blue" | "green" | "red" | "yellow";

type ProgressBarProps = {
  value?: number;
  size?: ProgressBarSize;
  color?: ProgressBarColor;
  label?: string;
  showValue?: boolean;
  class?: string;
};

const sizeStyles: { [key in ProgressBarSize]: string } = { sm: "h-1", md: "h-2", lg: "h-3" };

const colorStyles: { [key in ProgressBarColor]: string } = {
  blue: "bg-blue-500",
  green: "bg-green-500",
  red: "bg-red-500",
  yellow: "bg-yellow-500",
};

export const ProgressBar: Component<ProgressBarProps> = (props) => {
  const [local, _others] = splitProps(props, ["value", "size", "color", "label", "showValue", "class"]);
  const size = () => local.size ?? "md";
  const color = () => local.color ?? "blue";
  const isIndeterminate = () => local.value === undefined;
  const clampedValue = () => Math.min(100, Math.max(0, local.value ?? 0));

  return (
    <div class={local.class || ""}>
      <Show when={local.label || local.showValue}>
        <div class="flex justify-between mb-1">
          <Show when={local.label}>
            <span class="text-sm text-gray-300">{local.label}</span>
          </Show>
          <Show when={local.showValue && !isIndeterminate()}>
            <span class="text-sm text-gray-400">{Math.round(clampedValue())}%</span>
          </Show>
        </div>
      </Show>
      <div
        class={`w-full bg-gray-800 rounded-full overflow-hidden ${sizeStyles[size()]}`}
        role="progressbar"
        aria-valuenow={isIndeterminate() ? undefined : clampedValue()}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={local.label}>
        <div
          class={`${sizeStyles[size()]} ${colorStyles[color()]} rounded-full transition-all duration-300 ${
            isIndeterminate() ? "w-1/3 animate-[shimmer_1.5s_ease-in-out_infinite]" : ""
          }`}
          style={isIndeterminate() ? undefined : { width: `${clampedValue()}%` }} />
      </div>
    </div>
  );
};
