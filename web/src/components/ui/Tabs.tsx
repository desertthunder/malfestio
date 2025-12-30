import { createEffect, createSignal, For, Show, splitProps } from "solid-js";
import type { Accessor, Component, JSX } from "solid-js";

export type Tab = { id: string; label: string; icon?: JSX.Element; disabled?: boolean };

type TabsProps = {
  tabs: Tab[];
  activeTab?: string;
  onTabChange?: (tabId: string) => void;
  variant?: "line" | "contained";
  children?: (activeTab: Accessor<string>) => JSX.Element;
  class?: string;
};

// type TabsContextValue = { activeTab: Accessor<string>; setActiveTab: Setter<string> };

export const Tabs: Component<TabsProps> = (props) => {
  const [local, _others] = splitProps(props, ["tabs", "activeTab", "onTabChange", "variant", "children", "class"]);
  const variant = () => local.variant ?? "line";
  const [internalTab, setInternalTab] = createSignal("");
  const activeTab = () => local.activeTab ?? internalTab();

  createEffect(() => {
    if (local.tabs.length > 0) {
      setInternalTab(local.tabs[0].id);
    }
  });

  const selectTab = (tabId: string) => {
    if (local.onTabChange) {
      local.onTabChange(tabId);
    } else {
      setInternalTab(tabId);
    }
  };

  const handleKeyDown = (e: KeyboardEvent, currentIndex: number) => {
    const enabledTabs = local.tabs.filter((t) => !t.disabled);
    const currentEnabledIndex = enabledTabs.findIndex((t) => t.id === local.tabs[currentIndex].id);

    if (e.key === "ArrowRight" || e.key === "ArrowLeft") {
      e.preventDefault();
      const delta = e.key === "ArrowRight" ? 1 : -1;
      const nextIndex = (currentEnabledIndex + delta + enabledTabs.length) % enabledTabs.length;
      selectTab(enabledTabs[nextIndex].id);
    }
  };

  return (
    <div class={local.class || ""}>
      {/* Tab List */}
      <div
        role="tablist"
        class={`flex ${variant() === "line" ? "border-b border-gray-800" : "bg-gray-900 p-1 rounded-lg"}`}>
        <For each={local.tabs}>
          {(tab, index) => {
            const isActive = () => activeTab() === tab.id;
            const isDisabled = () => tab.disabled ?? false;

            return (
              <button
                role="tab"
                aria-selected={isActive()}
                aria-disabled={isDisabled()}
                tabIndex={isActive() ? 0 : -1}
                disabled={isDisabled()}
                onClick={() => !isDisabled() && selectTab(tab.id)}
                onKeyDown={(e) => handleKeyDown(e, index())}
                class={`
                  flex items-center gap-2 px-4 py-2 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-inset
                  ${isDisabled() ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}
                  ${
                  variant() === "line"
                    ? `border-b-2 -mb-px ${
                      isActive()
                        ? "border-blue-500 text-blue-400"
                        : "border-transparent text-gray-400 hover:text-white hover:border-gray-600"
                    }`
                    : `rounded-md ${
                      isActive() ? "bg-gray-700 text-white shadow" : "text-gray-400 hover:text-white hover:bg-gray-800"
                    }`
                }
                `}>
                <Show when={tab.icon}>
                  <span class="w-4 h-4">{tab.icon}</span>
                </Show>
                {tab.label}
              </button>
            );
          }}
        </For>
      </div>
      {/* Tab Content */}
      <Show when={local.children}>
        <div role="tabpanel" class="pt-4">{local.children!(activeTab)}</div>
      </Show>
    </div>
  );
};
