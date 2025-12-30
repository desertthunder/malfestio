import { createSignal, For, onCleanup, onMount, Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";
import { Motion, Presence } from "solid-motionone";

export type MenuItem = {
  id: string;
  label: string;
  icon?: JSX.Element;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
  onClick?: () => void;
};

export type MenuDivider = { type: "divider" };

export type MenuItemType = MenuItem | MenuDivider;

type MenuProps = { items: MenuItemType[]; trigger: JSX.Element; align?: "left" | "right"; class?: string };

const isDivider = (item: MenuItemType): item is MenuDivider => "type" in item && item.type === "divider";

export const Menu: Component<MenuProps> = (props) => {
  const [local, _others] = splitProps(props, ["items", "trigger", "align", "class"]);
  const align = () => local.align ?? "left";

  const [open, setOpen] = createSignal(false);
  const [focusIndex, setFocusIndex] = createSignal(-1);
  let containerRef: HTMLDivElement | undefined;

  const menuItems = () => local.items.filter((item): item is MenuItem => !isDivider(item));

  const handleClickOutside = (e: MouseEvent) => {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      setOpen(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!open()) return;

    const items = menuItems();
    if (e.key === "Escape") {
      setOpen(false);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusIndex((prev) => {
        let next = prev + 1;
        while (next < items.length && items[next].disabled) next++;
        return next < items.length ? next : prev;
      });
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusIndex((prev) => {
        let next = prev - 1;
        while (next >= 0 && items[next].disabled) next--;
        return next >= 0 ? next : prev;
      });
    } else if (e.key === "Enter" && focusIndex() >= 0) {
      e.preventDefault();
      const item = items[focusIndex()];
      if (item && !item.disabled) {
        item.onClick?.();
        setOpen(false);
      }
    }
  };

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener("click", handleClickOutside);
  });

  let itemIndex = 0;

  return (
    <div ref={containerRef} class={`relative inline-block ${local.class || ""}`} onKeyDown={handleKeyDown}>
      {/* Trigger */}
      <div
        onClick={() => {
          setOpen(!open());
          setFocusIndex(-1);
        }}
        class="cursor-pointer"
        aria-haspopup="menu"
        aria-expanded={open()}>
        {local.trigger}
      </div>

      {/* Menu */}
      <Presence>
        <Show when={open()}>
          <Motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.15 }}
            class={`absolute z-50 mt-1 min-w-48 bg-gray-800 border border-gray-700 rounded-lg shadow-xl py-1 ${
              align() === "right" ? "right-0" : "left-0"
            }`}
            role="menu">
            <For each={local.items}>
              {(item) => {
                if (isDivider(item)) {
                  return <div class="my-1 h-px bg-gray-700" role="separator" />;
                }

                const currentIndex = itemIndex;
                itemIndex++;
                const isFocused = () => focusIndex() === currentIndex;

                return (
                  <button
                    type="button"
                    role="menuitem"
                    disabled={item.disabled}
                    onClick={() => {
                      if (!item.disabled) {
                        item.onClick?.();
                        setOpen(false);
                      }
                    }}
                    onMouseEnter={() => setFocusIndex(currentIndex)}
                    class={`w-full flex items-center gap-3 px-4 py-2 text-sm text-left transition-colors
                      ${item.disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}
                      ${isFocused() ? "bg-gray-700" : ""}
                      ${item.danger ? "text-red-400 hover:bg-red-900/30" : "text-gray-300"}
                    `}>
                    <Show when={item.icon}>
                      <span class="w-4 h-4 flex items-center justify-center flex-shrink-0">{item.icon}</span>
                    </Show>
                    <span class="flex-1">{item.label}</span>
                    <Show when={item.shortcut}>
                      <kbd class="text-xs text-gray-500 font-mono">{item.shortcut}</kbd>
                    </Show>
                  </button>
                );
              }}
            </For>
          </Motion.div>
        </Show>
      </Presence>
    </div>
  );
};
