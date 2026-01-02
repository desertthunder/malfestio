import { useDensity } from "$lib/density-context";
import type { DensityMode } from "$lib/design-tokens";
import { splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

type ButtonVariant = "primary" | "secondary" | "danger" | "ghost";
type ButtonSize = "sm" | "md" | "lg";
type ButtonProps = { variant?: ButtonVariant; size?: ButtonSize; density?: DensityMode };

export const Button: Component<JSX.ButtonHTMLAttributes<HTMLButtonElement> & ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ["variant", "size", "density", "class", "children"]);
  const globalDensity = useDensity();
  const density = () => local.density || globalDensity;
  const base = () => local.size || "md";

  const variantClass = () => {
    switch (local.variant) {
      case "secondary":
        return "bg-gray-800 text-white hover:bg-gray-700";
      case "danger":
        return "bg-red-600 text-white hover:bg-red-500";
      case "ghost":
        return "bg-transparent text-gray-300 hover:bg-gray-800 hover:text-white";
      case "primary":
      default:
        return "bg-blue-600 text-white hover:bg-blue-500";
    }
  };

  const sizeClass = () => {
    const densityMode = density();
    const baseSize = base();
    if (baseSize === "sm") {
      return densityMode === "compact"
        ? "px-2 py-1 text-xs"
        : densityMode === "spacious"
        ? "px-4 py-2 text-sm"
        : "px-3 py-1.5 text-sm";
    }

    if (baseSize === "lg") {
      return densityMode === "compact"
        ? "px-5 py-2 text-base"
        : densityMode === "spacious"
        ? "px-8 py-4 text-xl"
        : "px-6 py-3 text-lg";
    }

    return densityMode === "compact"
      ? "px-3 py-1.5 text-sm"
      : densityMode === "spacious"
      ? "px-6 py-3 text-base"
      : "px-4 py-2";
  };

  return (
    <button
      class={`
        inline-flex items-center justify-center rounded-sm transition-standard font-medium
        focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900
        disabled:opacity-50 disabled:cursor-not-allowed
        ${variantClass()}
        ${sizeClass()}
        ${local.class || ""}
      `}
      {...others}>
      {local.children}
    </button>
  );
};
