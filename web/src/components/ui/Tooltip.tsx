import { createSignal, onCleanup, Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

type TooltipPosition = "top" | "right" | "bottom" | "left";

type TooltipProps = {
  content: string | JSX.Element;
  position?: TooltipPosition;
  delay?: number;
  children: JSX.Element;
  class?: string;
};

const positionStyles: Record<TooltipPosition, { tooltip: string; arrow: string }> = {
  top: {
    tooltip: "bottom-full left-1/2 -translate-x-1/2 mb-2",
    arrow: "top-full left-1/2 -translate-x-1/2 border-t-gray-800 border-x-transparent border-b-transparent",
  },
  bottom: {
    tooltip: "top-full left-1/2 -translate-x-1/2 mt-2",
    arrow: "bottom-full left-1/2 -translate-x-1/2 border-b-gray-800 border-x-transparent border-t-transparent",
  },
  left: {
    tooltip: "right-full top-1/2 -translate-y-1/2 mr-2",
    arrow: "left-full top-1/2 -translate-y-1/2 border-l-gray-800 border-y-transparent border-r-transparent",
  },
  right: {
    tooltip: "left-full top-1/2 -translate-y-1/2 ml-2",
    arrow: "right-full top-1/2 -translate-y-1/2 border-r-gray-800 border-y-transparent border-l-transparent",
  },
};

export const Tooltip: Component<TooltipProps> = (props) => {
  const [local, _others] = splitProps(props, ["content", "position", "delay", "children", "class"]);
  const position = () => local.position ?? "top";
  const delay = () => local.delay ?? 200;

  const [visible, setVisible] = createSignal(false);
  let timeoutId: number | undefined;

  const show = () => {
    timeoutId = window.setTimeout(() => setVisible(true), delay());
  };

  const hide = () => {
    if (timeoutId) clearTimeout(timeoutId);
    setVisible(false);
  };

  onCleanup(() => {
    if (timeoutId) clearTimeout(timeoutId);
  });

  return (
    <span
      class={`relative inline-flex ${local.class || ""}`}
      onMouseEnter={show}
      onMouseLeave={hide}
      onFocus={show}
      onBlur={hide}>
      {local.children}
      <Show when={visible()}>
        <span
          role="tooltip"
          class={`absolute z-50 px-2.5 py-1.5 text-xs text-white bg-gray-800 border border-gray-700 rounded shadow-lg whitespace-nowrap ${
            positionStyles[position()].tooltip
          }`}>
          {local.content}
          <span class={`absolute w-0 h-0 border-4 ${positionStyles[position()].arrow}`} aria-hidden="true" />
        </span>
      </Show>
    </span>
  );
};

export const KeyboardHint: Component<{ keys: string[]; children: JSX.Element }> = (props) => {
  const keysDisplay = () =>
    props.keys.map((key) => (
      <kbd class="px-1.5 py-0.5 bg-gray-700 border border-gray-600 rounded text-xs font-mono">{key}</kbd>
    ));

  return <Tooltip content={<span class="flex items-center gap-1">{keysDisplay()}</span>}>{props.children}</Tooltip>;
};
