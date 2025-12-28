import { splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "danger" | "ghost";
  size?: "sm" | "md" | "lg";
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ["variant", "size", "class", "children"]);

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
    switch (local.size) {
      case "sm":
        return "px-3 py-1.5 text-sm";
      case "lg":
        return "px-6 py-3 text-lg";
      case "md":
      default:
        return "px-4 py-2";
    }
  };

  return (
    <button
      class={`
        inline-flex items-center justify-center rounded-sm transition-colors duration-200 font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-900 disabled:opacity-50 disabled:cursor-not-allowed
        ${variantClass()}
        ${sizeClass()}
        ${local.class || ""}
      `}
      {...others}>
      {local.children}
    </button>
  );
};
