import { type Component, For } from "solid-js";

type ToolbarButton = { id: string; icon: string; label: string; shortcut?: string; action: () => void };

type EditorToolbarProps = {
  onBold: () => void;
  onItalic: () => void;
  onHeading: (level: 1 | 2 | 3 | 4 | 5 | 6) => void;
  onLink: () => void;
  onCode: () => void;
  onCodeBlock: () => void;
  onWikilink: () => void;
  onList: () => void;
  class?: string;
};

const ToolbarButtonComponent: Component<{ btn: ToolbarButton }> = (props) => (
  <button
    type="button"
    onClick={() => props.btn.action()}
    title={props.btn.shortcut ? `${props.btn.label} (${props.btn.shortcut})` : props.btn.label}
    class="p-2 rounded hover:bg-slate-700 text-slate-400 hover:text-white transition-colors">
    <span class={`${props.btn.icon} text-lg`} />
  </button>
);

export const EditorToolbar: Component<EditorToolbarProps> = (props) => {
  const buttons: ToolbarButton[] = [
    {
      id: "bold",
      icon: "i-ri-bold",
      label: "Bold",
      shortcut: "⌘B",
      action: () => props.onBold(),
    },
    { id: "italic", icon: "i-ri-italic", label: "Italic", shortcut: "⌘I", action: () => props.onItalic() },
    {
      id: "h1",
      icon: "i-ri-h-1",
      label: "Heading 1",
      action: () => props.onHeading(1),
    },
    { id: "h2", icon: "i-ri-h-2", label: "Heading 2", action: () => props.onHeading(2) },
    { id: "h3", icon: "i-ri-h-3", label: "Heading 3", action: () => props.onHeading(3) },
    { id: "h4", icon: "i-ri-h-4", label: "Heading 4", action: () => props.onHeading(4) },
    {
      id: "h5",
      icon: "i-ri-h-5",
      label: "Heading 5",
      action: () => props.onHeading(5),
    },
    { id: "h6", icon: "i-ri-h-6", label: "Heading 6", action: () => props.onHeading(6) },
    {
      id: "link",
      icon: "i-ri-link",
      label: "Link",
      shortcut: "⌘K",
      action: () => props.onLink(),
    },
    { id: "code", icon: "i-ri-code-line", label: "Inline Code", action: () => props.onCode() },
    { id: "codeblock", icon: "i-ri-code-box-line", label: "Code Block", action: () => props.onCodeBlock() },
    { id: "wikilink", icon: "i-ri-links-line", label: "Wikilink", action: () => props.onWikilink() },
    {
      id: "list",
      icon: "i-ri-list-unordered",
      label: "Bullet List",
      action: () => props.onList(),
    },
  ];

  return (
    <div class={`flex items-center gap-1 p-2 bg-slate-800 border-b border-slate-700 rounded-t-lg ${props.class ?? ""}`}>
      <div class="flex items-center">
        <ToolbarButtonComponent btn={buttons[0]} />
        <ToolbarButtonComponent btn={buttons[1]} />
      </div>
      <div class="w-px h-6 bg-slate-600 mx-1" />
      <div class="flex items-center">
        <For each={buttons.slice(2, 8)}>{(btn) => <ToolbarButtonComponent btn={btn} />}</For>
      </div>
      <div class="w-px h-6 bg-slate-600 mx-1" />
      <div class="flex items-center">
        <For each={buttons.slice(8, 12)}>{(btn) => <ToolbarButtonComponent btn={btn} />}</For>
      </div>
      <div class="w-px h-6 bg-slate-600 mx-1" />
      <div class="flex items-center">
        <ToolbarButtonComponent btn={buttons[12]} />
      </div>
    </div>
  );
};

export default EditorToolbar;
