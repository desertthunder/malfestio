import { createMemo, createSignal, For, Show, splitProps } from "solid-js";
import type { Accessor, Component, JSX } from "solid-js";

export type Column<T> = {
  key: keyof T | string;
  header: string;
  sortable?: boolean;
  render?: (row: T, index: number) => JSX.Element;
  width?: string;
};

type DataTableProps<T> = {
  columns: Column<T>[];
  data: T[];
  getRowId: (row: T) => string;
  selectable?: boolean;
  expandable?: (row: T) => JSX.Element | null;
  onSelectionChange?: (selectedIds: string[]) => void;
  class?: string;
};

type SortDirection = "asc" | "desc" | null;

const SortIcon: Component<{ direction: SortDirection }> = (props) => (
  <svg class="w-4 h-4 ml-1 inline-block" fill="none" viewBox="0 0 24 24" stroke="currentColor">
    <Show when={props.direction === "asc"}>
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
    </Show>
    <Show when={props.direction === "desc"}>
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
    </Show>
    <Show when={!props.direction}>
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l4-4 4 4M8 15l4 4 4-4" />
    </Show>
  </svg>
);

export function DataTable<T>(props: DataTableProps<T>): JSX.Element {
  const [local, _others] = splitProps(props, [
    "columns",
    "data",
    "getRowId",
    "selectable",
    "expandable",
    "onSelectionChange",
    "class",
  ]);

  const [sortKey, setSortKey] = createSignal<string | null>(null);
  const [sortDir, setSortDir] = createSignal<SortDirection>(null);
  const [selected, setSelected] = createSignal<Set<string>>(new Set());
  const [expanded, setExpanded] = createSignal<Set<string>>(new Set());

  const sortedData: Accessor<T[]> = createMemo(() => {
    const key = sortKey();
    const dir = sortDir();
    if (!key || !dir) return local.data;

    return [...local.data].sort((a, b) => {
      const aVal = (a as Record<string, unknown>)[key];
      const bVal = (b as Record<string, unknown>)[key];
      if (aVal === bVal) return 0;
      if (aVal == null) return 1;
      if (bVal == null) return -1;
      const cmp = aVal < bVal ? -1 : 1;
      return dir === "asc" ? cmp : -cmp;
    });
  });

  const handleSort = (key: string) => {
    if (sortKey() === key) {
      setSortDir((d) => (d === "asc" ? "desc" : d === "desc" ? null : "asc"));
      if (sortDir() === null) setSortKey(null);
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const toggleSelect = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      local.onSelectionChange?.([...next]);
      return next;
    });
  };

  const toggleSelectAll = () => {
    if (selected().size === local.data.length) {
      setSelected(new Set<string>());
      local.onSelectionChange?.([]);
    } else {
      const all = new Set(local.data.map(local.getRowId));
      setSelected(all);
      local.onSelectionChange?.([...all]);
    }
  };

  const toggleExpand = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const getCellValue = (row: T, col: Column<T>, index: number): JSX.Element => {
    if (col.render) return col.render(row, index);
    const value = (row as Record<string, unknown>)[col.key as string];
    return <>{value != null ? String(value) : ""}</>;
  };

  return (
    <div class={`overflow-x-auto ${local.class || ""}`}>
      <table class="w-full text-sm text-left">
        <thead class="text-xs text-gray-400 uppercase bg-gray-900 border-b border-gray-700">
          <tr>
            <Show when={local.expandable}>
              <th class="w-8 px-2 py-3" />
            </Show>
            <Show when={local.selectable}>
              <th class="w-8 px-2 py-3">
                <input
                  type="checkbox"
                  checked={selected().size === local.data.length && local.data.length > 0}
                  onChange={toggleSelectAll}
                  class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500" />
              </th>
            </Show>
            <For each={local.columns}>
              {(col) => (
                <th
                  class={`px-4 py-3 ${col.sortable ? "cursor-pointer hover:bg-gray-800 select-none" : ""}`}
                  style={{ width: col.width }}
                  onClick={() => col.sortable && handleSort(col.key as string)}>
                  <span class="flex items-center">
                    {col.header}
                    <Show when={col.sortable}>
                      <SortIcon direction={sortKey() === col.key ? sortDir() : null} />
                    </Show>
                  </span>
                </th>
              )}
            </For>
          </tr>
        </thead>
        <tbody>
          <For each={sortedData()}>
            {(row, index) => {
              const id = local.getRowId(row);
              const isExpanded = () => expanded().has(id);
              const expandedContent = () => local.expandable?.(row);

              return (
                <>
                  <tr class="border-b border-gray-800 hover:bg-gray-800/50 text-gray-300">
                    <Show when={local.expandable}>
                      <td class="px-2 py-3">
                        <Show when={expandedContent()}>
                          <button
                            onClick={() => toggleExpand(id)}
                            class="p-1 hover:bg-gray-700 rounded"
                            aria-expanded={isExpanded()}
                            aria-label={isExpanded() ? "Collapse row" : "Expand row"}>
                            <svg
                              class={`w-4 h-4 transition-transform ${isExpanded() ? "rotate-90" : ""}`}
                              fill="none"
                              viewBox="0 0 24 24"
                              stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                            </svg>
                          </button>
                        </Show>
                      </td>
                    </Show>
                    <Show when={local.selectable}>
                      <td class="px-2 py-3">
                        <input
                          type="checkbox"
                          checked={selected().has(id)}
                          onChange={() => toggleSelect(id)}
                          class="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500" />
                      </td>
                    </Show>
                    <For each={local.columns}>
                      {(col) => <td class="px-4 py-3">{getCellValue(row, col, index())}</td>}
                    </For>
                  </tr>
                  <Show when={isExpanded() && expandedContent()}>
                    <tr class="bg-gray-900/50">
                      <td
                        colSpan={local.columns.length + (local.selectable ? 1 : 0) + (local.expandable ? 1 : 0)}
                        class="px-4 py-3">
                        {expandedContent()}
                      </td>
                    </tr>
                  </Show>
                </>
              );
            }}
          </For>
        </tbody>
      </table>
    </div>
  );
}
