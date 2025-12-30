import { onCleanup, onMount, Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";
import { Motion, Presence } from "solid-motionone";

type RightPanelProps = {
  open: boolean;
  onClose: () => void;
  title?: string;
  width?: string;
  children: JSX.Element;
  class?: string;
};

export const RightPanel: Component<RightPanelProps> = (props) => {
  const [local, _others] = splitProps(props, ["open", "onClose", "title", "width", "children", "class"]);
  const width = () => local.width ?? "400px";

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape" && local.open) {
      local.onClose();
    }
  };

  onMount(() => {
    document.addEventListener("keydown", handleKeyDown);
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleKeyDown);
  });

  return (
    <Presence>
      <Show when={local.open}>
        {/* Backdrop */}
        <Motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
          class="fixed inset-0 z-40 bg-black/40"
          onClick={local.onClose}
          aria-hidden="true" />
        {/* Panel */}
        <Motion.div
          initial={{ x: "100%" }}
          animate={{ x: 0 }}
          exit={{ x: "100%" }}
          transition={{ duration: 0.25, easing: [0.22, 1, 0.36, 1] }}
          class={`fixed top-0 right-0 bottom-0 z-50 bg-gray-900 border-l border-gray-800 shadow-2xl flex flex-col ${
            local.class || ""
          }`}
          style={{ width: width() }}
          role="dialog"
          aria-modal="true"
          aria-label={local.title || "Panel"}>
          {/* Header */}
          <div class="h-16 flex items-center justify-between px-6 border-b border-gray-800">
            <Show when={local.title}>
              <h2 class="text-lg font-semibold text-white">{local.title}</h2>
            </Show>
            <button
              onClick={() => local.onClose()}
              class="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors ml-auto"
              aria-label="Close panel">
              <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
          {/* Content */}
          <div class="flex-1 overflow-y-auto p-6 text-gray-300">{local.children}</div>
        </Motion.div>
      </Show>
    </Presence>
  );
};
