import { Show, splitProps } from "solid-js";
import type { Component } from "solid-js";

type AvatarSize = "xs" | "sm" | "md" | "lg" | "xl";

type AvatarProps = { src?: string; alt?: string; name?: string; size?: AvatarSize; class?: string };

const sizeStyles = {
  xs: "w-6 h-6 text-xs",
  sm: "w-8 h-8 text-xs",
  md: "w-10 h-10 text-sm",
  lg: "w-12 h-12 text-base",
  xl: "w-16 h-16 text-lg",
};

const getInitials = (name: string): string => {
  return name.split(" ").map((part) => part[0]).join("").slice(0, 2).toUpperCase();
};

const stringToColor = (str: string): string => {
  const colors = [
    "bg-blue-600",
    "bg-green-600",
    "bg-purple-600",
    "bg-pink-600",
    "bg-indigo-600",
    "bg-teal-600",
    "bg-orange-600",
    "bg-cyan-600",
  ];
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = str.charCodeAt(i) + ((hash << 5) - hash);
  }
  return colors[Math.abs(hash) % colors.length];
};

export const Avatar: Component<AvatarProps> = (props) => {
  const [local, others] = splitProps(props, ["src", "alt", "name", "size", "class"]);
  const size = () => local.size ?? "md";
  const initials = () => (local.name ? getInitials(local.name) : "?");
  const bgColor = () => (local.name ? stringToColor(local.name) : "bg-gray-700");

  return (
    <div
      class={`relative inline-flex items-center justify-center rounded-full overflow-hidden flex-shrink-0 ${
        sizeStyles[size()]
      } ${local.class || ""}`}
      {...others}>
      <Show
        when={local.src}
        fallback={
          <span class={`w-full h-full flex items-center justify-center font-semibold text-white ${bgColor()}`}>
            {initials()}
          </span>
        }>
        <img
          src={local.src}
          alt={local.alt || local.name || "Avatar"}
          class="w-full h-full object-cover"
          onError={(e) => (e.currentTarget as HTMLImageElement).style.display = "none"} />
      </Show>
    </div>
  );
};
