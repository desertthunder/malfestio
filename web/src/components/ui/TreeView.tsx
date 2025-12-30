import { createSignal, For, Show, splitProps } from "solid-js";
import type { Component, JSX } from "solid-js";

export type TreeNode = { id: string; label: string; icon?: JSX.Element; children?: TreeNode[] };

type TreeViewProps = { nodes: TreeNode[]; onSelect?: (node: TreeNode) => void; class?: string };

type TreeNodeItemProps = { node: TreeNode; level: number; onSelect?: (node: TreeNode) => void };

const ChevronIcon: Component<{ expanded: boolean }> = (props) => (
  <svg
    class={`w-4 h-4 transition-transform duration-200 ${props.expanded ? "rotate-90" : ""}`}
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor">
    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
  </svg>
);

const TreeNodeItem: Component<TreeNodeItemProps> = (props) => {
  const [expanded, setExpanded] = createSignal(false);
  const hasChildren = () => props.node.children && props.node.children.length > 0;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      if (hasChildren()) {
        setExpanded(!expanded());
      }
      props.onSelect?.(props.node);
    } else if (e.key === "ArrowRight" && hasChildren() && !expanded()) {
      setExpanded(true);
    } else if (e.key === "ArrowLeft" && expanded()) {
      setExpanded(false);
    }
  };

  return (
    <li role="treeitem" aria-expanded={hasChildren() ? expanded() : undefined}>
      <div
        class="flex items-center gap-1 px-2 py-1.5 hover:bg-gray-800 cursor-pointer text-gray-300 hover:text-white transition-colors rounded"
        style={{ "padding-left": `${props.level * 16 + 8}px` }}
        onClick={() => {
          if (hasChildren()) setExpanded(!expanded());
          props.onSelect?.(props.node);
        }}
        onKeyDown={handleKeyDown}
        tabIndex={0}
        role="button">
        <span class="w-4 h-4 flex items-center justify-center text-gray-500">
          <Show when={hasChildren()} fallback={<span class="w-4" />}>
            <ChevronIcon expanded={expanded()} />
          </Show>
        </span>
        <Show when={props.node.icon}>
          <span class="w-4 h-4 flex items-center justify-center">{props.node.icon}</span>
        </Show>
        <span class="text-sm truncate">{props.node.label}</span>
      </div>
      <Show when={expanded() && hasChildren()}>
        <ul role="group" class="border-l border-gray-800 ml-4">
          <For each={props.node.children}>
            {(child) => <TreeNodeItem node={child} level={props.level + 1} onSelect={props.onSelect} />}
          </For>
        </ul>
      </Show>
    </li>
  );
};

export const TreeView: Component<TreeViewProps> = (props) => {
  const [local, others] = splitProps(props, ["nodes", "onSelect", "class"]);

  return (
    <ul role="tree" class={`text-sm ${local.class || ""}`} {...others}>
      <For each={local.nodes}>{(node) => <TreeNodeItem node={node} level={0} onSelect={local.onSelect} />}</For>
    </ul>
  );
};
