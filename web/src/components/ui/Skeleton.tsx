import { For, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

type SkeletonSize = "sm" | "md" | "lg";
type SkeletonRounded = "none" | SkeletonSize | "full";

type SkeletonProps = { width?: string; height?: string; rounded?: SkeletonRounded };

const roundedClass: Record<string, string> = {
  none: "",
  sm: "rounded-sm",
  md: "rounded-md",
  lg: "rounded-lg",
  full: "rounded-full",
};

export const Skeleton: Component<JSX.HTMLAttributes<HTMLDivElement> & SkeletonProps> = (props) => {
  const [local, others] = splitProps(props, ["width", "height", "rounded", "class"]);
  const rounded = () => local.rounded ?? "md";

  return (
    <div
      class={`animate-pulse bg-gray-800 ${roundedClass[rounded()]} ${local.class || ""}`}
      style={{ width: local.width, height: local.height || "1rem" }}
      aria-hidden="true"
      {...others} />
  );
};

/** Skeleton text line */
export const SkeletonText: Component<{ lines?: number; class?: string }> = (props) => {
  const lines = () => props.lines ?? 3;

  return (
    <div class={`space-y-2 ${props.class || ""}`}>
      <For each={Array.from({ length: lines() })}>
        {(line) => <Skeleton width={line === lines() - 1 ? "75%" : "100%"} height="0.875rem" />}
      </For>
    </div>
  );
};

export const SkeletonAvatar: Component<{ size?: "sm" | "md" | "lg" }> = (props) => {
  const sizes = { sm: "32px", md: "48px", lg: "64px" };
  const size = () => sizes[props.size ?? "md"];
  return <Skeleton width={size()} height={size()} rounded="full" />;
};
