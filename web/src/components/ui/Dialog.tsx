import { onCleanup, onMount, Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

export type DialogVariant = "transactional" | "danger" | "passive";

type DialogProps = {
  open: boolean;
  onClose: () => void;
  title: string;
  variant?: DialogVariant;
  children: JSX.Element;
  actions?: JSX.Element;
};

const variantStyles: Record<DialogVariant, { header: string; primary: string }> = {
  transactional: { header: "border-b border-gray-700", primary: "bg-blue-600 hover:bg-blue-500" },
  danger: { header: "border-b border-red-900/50", primary: "bg-red-600 hover:bg-red-500" },
  passive: { header: "border-b border-gray-700", primary: "bg-gray-600 hover:bg-gray-500" },
};

export const Dialog: Component<DialogProps> = (props) => {
  const [local, _others] = splitProps(props, ["open", "onClose", "title", "variant", "children", "actions"]);
  const variant = () => local.variant ?? "transactional";
  let dialogRef: HTMLDivElement | undefined;

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
    <Show when={local.open}>
      <div
        class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm animate-fade-in"
        onClick={() => local.onClose()}
        aria-hidden="true" />

      <div
        class="fixed inset-0 z-50 flex items-center justify-center p-4 animate-scale-in"
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title">
        <div
          ref={dialogRef}
          class="w-full max-w-md bg-gray-900 border border-gray-700 shadow-2xl"
          onClick={(e) => e.stopPropagation()}>
          <div class={`px-6 py-4 ${variantStyles[variant()].header}`}>
            <h2 id="dialog-title" class="text-lg font-semibold text-white">{local.title}</h2>
          </div>
          <div class="px-6 py-4 text-gray-300">{local.children}</div>
          <Show when={local.actions}>
            <div class="px-6 py-4 bg-gray-950/50 flex justify-end gap-3">{local.actions}</div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
