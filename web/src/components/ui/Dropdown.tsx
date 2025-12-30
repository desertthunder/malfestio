import { createSignal, For, onCleanup, onMount, Show, splitProps } from "solid-js";
import type { Component } from "solid-js";
import { Motion, Presence } from "solid-motionone";

export type DropdownOption = { value: string; label: string; disabled?: boolean };

type DropdownProps = {
  options: DropdownOption[];
  value?: string | string[];
  onChange?: (value: string | string[]) => void;
  placeholder?: string;
  multiple?: boolean;
  searchable?: boolean;
  disabled?: boolean;
  class?: string;
};

const ChevronIcon: Component<{ open: boolean }> = (props) => (
  <svg
    class={`w-4 h-4 transition-transform ${props.open ? "rotate-180" : ""}`}
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
  </svg>
);

export const Dropdown: Component<DropdownProps> = (props) => {
  const [local, _others] = splitProps(props, [
    "options",
    "value",
    "onChange",
    "placeholder",
    "multiple",
    "searchable",
    "disabled",
    "class",
  ]);

  const [open, setOpen] = createSignal(false);
  const [search, setSearch] = createSignal("");
  const [focusIndex, setFocusIndex] = createSignal(-1);
  let containerRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;

  const selectedValues = (): string[] => {
    if (!local.value) return [];
    return Array.isArray(local.value) ? local.value : [local.value];
  };

  const filteredOptions = () => {
    const q = search().toLowerCase();
    if (!q) return local.options;
    return local.options.filter((o) => o.label.toLowerCase().includes(q));
  };

  const displayLabel = () => {
    const vals = selectedValues();
    if (vals.length === 0) return local.placeholder || "Select...";
    if (vals.length === 1) {
      return local.options.find((o) => o.value === vals[0])?.label || vals[0];
    }
    return `${vals.length} selected`;
  };

  const toggleOption = (value: string) => {
    if (local.multiple) {
      const current = selectedValues();
      const next = current.includes(value) ? current.filter((v) => v !== value) : [...current, value];
      local.onChange?.(next);
    } else {
      local.onChange?.(value);
      setOpen(false);
    }
    setSearch("");
  };

  const handleClickOutside = (e: MouseEvent) => {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      setOpen(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    const opts = filteredOptions();
    if (e.key === "Escape") {
      setOpen(false);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      setFocusIndex((prev) => Math.min(prev + 1, opts.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setFocusIndex((prev) => Math.max(prev - 1, 0));
    } else if (e.key === "Enter" && focusIndex() >= 0) {
      e.preventDefault();
      const opt = opts[focusIndex()];
      if (opt && !opt.disabled) toggleOption(opt.value);
    }
  };

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener("click", handleClickOutside);
  });

  return (
    <div ref={containerRef} class={`relative ${local.class || ""}`} onKeyDown={handleKeyDown}>
      {/* Trigger */}
      <button
        type="button"
        disabled={local.disabled}
        onClick={() => {
          setOpen(!open());
          if (!open() && inputRef) inputRef.focus();
        }}
        class={`w-full flex items-center justify-between gap-2 px-4 py-2 bg-gray-800 border border-gray-700 text-left text-sm transition-colors rounded
          ${
          local.disabled
            ? "opacity-50 cursor-not-allowed"
            : "hover:border-gray-600 focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
        }
          ${open() ? "border-blue-500 ring-1 ring-blue-500" : ""}
        `}
        aria-haspopup="listbox"
        aria-expanded={open()}>
        <span class={`truncate ${selectedValues().length === 0 ? "text-gray-500" : "text-white"}`}>
          {displayLabel()}
        </span>
        <ChevronIcon open={open()} />
      </button>

      {/* Dropdown */}
      <Presence>
        <Show when={open()}>
          <Motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.15 }}
            class="absolute z-50 mt-1 w-full bg-gray-800 border border-gray-700 rounded shadow-xl max-h-60 overflow-hidden">
            {/* Search */}
            <Show when={local.searchable}>
              <div class="p-2 border-b border-gray-700">
                <input
                  ref={inputRef}
                  type="text"
                  value={search()}
                  onInput={(e) => {
                    setSearch(e.currentTarget.value);
                    setFocusIndex(0);
                  }}
                  placeholder="Search..."
                  class="w-full px-3 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-white placeholder-gray-500 focus:outline-none focus:border-blue-500" />
              </div>
            </Show>

            {/* Options */}
            <ul role="listbox" class="overflow-y-auto max-h-48 py-1">
              <For each={filteredOptions()}>
                {(option, index) => {
                  const isSelected = () => selectedValues().includes(option.value);
                  const isFocused = () => focusIndex() === index();

                  return (
                    <li
                      role="option"
                      aria-selected={isSelected()}
                      aria-disabled={option.disabled}
                      onClick={() => !option.disabled && toggleOption(option.value)}
                      onMouseEnter={() => setFocusIndex(index())}
                      class={`flex items-center gap-2 px-4 py-2 text-sm cursor-pointer transition-colors
                        ${option.disabled ? "opacity-50 cursor-not-allowed" : ""}
                        ${isFocused() ? "bg-gray-700" : ""}
                        ${isSelected() ? "text-blue-400" : "text-gray-300"}
                      `}>
                      <Show when={local.multiple}>
                        <input
                          type="checkbox"
                          checked={isSelected()}
                          disabled={option.disabled}
                          class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600"
                          tabIndex={-1} />
                      </Show>
                      <span class="truncate">{option.label}</span>
                      <Show when={isSelected() && !local.multiple}>
                        <svg class="w-4 h-4 ml-auto" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                      </Show>
                    </li>
                  );
                }}
              </For>
              <Show when={filteredOptions().length === 0}>
                <li class="px-4 py-2 text-sm text-gray-500">No options found</li>
              </Show>
            </ul>
          </Motion.div>
        </Show>
      </Presence>
    </div>
  );
};
