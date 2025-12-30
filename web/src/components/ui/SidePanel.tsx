import { createSignal, For, Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

export type NavItem = { id: string; label: string; icon?: JSX.Element; href?: string; onClick?: () => void };

type SidePanelProps = {
  items: NavItem[];
  activeId?: string;
  collapsed?: boolean;
  onToggle?: () => void;
  header?: JSX.Element;
  footer?: JSX.Element;
  class?: string;
};

const MenuIcon: Component = () => (
  <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
  </svg>
);

export const SidePanel: Component<SidePanelProps> = (props) => {
  const [local, _others] = splitProps(props, [
    "items",
    "activeId",
    "collapsed",
    "onToggle",
    "header",
    "footer",
    "class",
  ]);
  const [internalCollapsed, setInternalCollapsed] = createSignal(false);
  const isCollapsed = () => local.collapsed ?? internalCollapsed();

  const toggle = () => {
    if (local.onToggle) {
      local.onToggle();
    } else {
      setInternalCollapsed(!internalCollapsed());
    }
  };

  return (
    <aside
      class={`flex flex-col bg-gray-900 border-r border-gray-800 transition-all duration-200 ${
        isCollapsed() ? "w-16" : "w-64"
      } ${local.class || ""}`}>
      {/* Header */}
      <div class="h-16 flex items-center justify-between px-4 border-b border-gray-800">
        <Show when={!isCollapsed() && local.header}>{local.header}</Show>
        <button
          onClick={toggle}
          class="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded transition-colors"
          aria-label={isCollapsed() ? "Expand sidebar" : "Collapse sidebar"}>
          <MenuIcon />
        </button>
      </div>

      {/* Navigation */}
      <nav class="flex-1 py-4 overflow-y-auto" role="navigation">
        <ul class="space-y-1 px-2">
          <For each={local.items}>
            {(item) => {
              const isActive = () => local.activeId === item.id;
              const Component = item.href ? "a" : "button";

              return (
                <li>
                  <Component
                    href={item.href}
                    onClick={item.onClick}
                    class={`w-full flex items-center gap-3 px-3 py-2 rounded transition-colors text-sm ${
                      isActive() ? "bg-blue-600/20 text-blue-400" : "text-gray-400 hover:text-white hover:bg-gray-800"
                    }`}
                    aria-current={isActive() ? "page" : undefined}>
                    <Show when={item.icon}>
                      <span class="w-5 h-5 flex items-center justify-center flex-shrink-0">{item.icon}</span>
                    </Show>
                    <Show when={!isCollapsed()}>
                      <span class="truncate">{item.label}</span>
                    </Show>
                  </Component>
                </li>
              );
            }}
          </For>
        </ul>
      </nav>

      {/* Footer */}
      <Show when={local.footer}>
        <div class="border-t border-gray-800 px-4 py-4">
          <Show when={!isCollapsed()}>{local.footer}</Show>
        </div>
      </Show>
    </aside>
  );
};
