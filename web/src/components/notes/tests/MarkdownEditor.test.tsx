import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MarkdownEditor, type MarkdownEditorAPI } from "../MarkdownEditor";

vi.mock("shiki", () => ({ codeToHtml: vi.fn(async (code: string) => `<pre><code>${code}</code></pre>`) }));

describe("MarkdownEditor", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  describe("Basic Rendering", () => {
    it("renders editor with initial value", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="# Hello" onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      expect(textarea).toBeInTheDocument();
      expect(textarea.value).toBe("# Hello");
    });

    it("renders placeholder when provided", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="" onChange={onChange} placeholder="Write here..." />);

      const textarea = screen.getByPlaceholderText("Write here...");
      expect(textarea).toBeInTheDocument();
    });

    it("renders line numbers by default", () => {
      const [value, setValue] = createSignal("line1\nline2\nline3");
      render(() => <MarkdownEditor value={value()} onChange={setValue} />);

      expect(screen.getByText("1")).toBeInTheDocument();
      expect(screen.getByText("2")).toBeInTheDocument();
      expect(screen.getByText("3")).toBeInTheDocument();
    });

    it("hides line numbers when showLineNumbers is false", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="line1\nline2" onChange={onChange} showLineNumbers={false} />);

      expect(screen.queryByText("1")).not.toBeInTheDocument();
      expect(screen.queryByText("2")).not.toBeInTheDocument();
    });
  });

  describe("Input Handling", () => {
    it("calls onChange when user types", async () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="" onChange={onChange} />);

      const textarea = screen.getByRole("textbox");
      fireEvent.input(textarea, { target: { value: "new text" } });

      expect(onChange).toHaveBeenCalledWith("new text");
    });

    it("updates value when controlled externally", () => {
      const [value, setValue] = createSignal("initial");
      render(() => <MarkdownEditor value={value()} onChange={setValue} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      expect(textarea.value).toBe("initial");

      setValue("updated");
      expect(textarea.value).toBe("updated");
    });
  });

  describe("Cursor Positioning", () => {
    it("maintains selectionStart and selectionEnd on textarea", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="Hello World" onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      textarea.setSelectionRange(0, 5);

      expect(textarea.selectionStart).toBe(0);
      expect(textarea.selectionEnd).toBe(5);
    });

    it("preserves cursor position when onChange updates value", async () => {
      const onChange = vi.fn();

      render(() => <MarkdownEditor value="Hello" onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;

      textarea.focus();
      textarea.setSelectionRange(5, 5);
      fireEvent.input(textarea, { target: { value: "Hello!" } });

      expect(onChange).toHaveBeenCalled();
    });

    it("selection range preserved during typing", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="ABC" onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      textarea.setSelectionRange(0, 3);

      expect(textarea.selectionStart).toBe(0);
      expect(textarea.selectionEnd).toBe(3);
    });
  });

  describe("IME Composition Handling", () => {
    let editorApi: MarkdownEditorAPI | undefined;

    beforeEach(() => {
      editorApi = undefined;
    });

    it("blocks highlight updates during composition", async () => {
      const onChange = vi.fn();
      render(() => (
        <MarkdownEditor
          value=""
          onChange={onChange}
          ref={(api) => {
            editorApi = api;
            void editorApi;
          }} />
      ));

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;

      fireEvent.compositionStart(textarea);
      fireEvent.input(textarea, { target: { value: "中" } });
      expect(onChange).toHaveBeenCalledWith("中");
    });

    it("triggers highlight after compositionend", async () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="" onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;

      fireEvent.compositionStart(textarea);
      fireEvent.input(textarea, { target: { value: "日本語" } });
      fireEvent.compositionEnd(textarea);
      expect(onChange).toHaveBeenCalledWith("日本語");
    });

    it("cursor position stable during IME input", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="prefix " onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      textarea.setSelectionRange(7, 7);
      fireEvent.compositionStart(textarea);
      expect(textarea.selectionStart).toBe(7);
    });
  });

  describe("Toolbar Integration via API", () => {
    let editorApi: MarkdownEditorAPI | undefined;

    beforeEach(() => {
      editorApi = undefined;
    });

    it("exposes insertAtCursor via ref callback", () => {
      const onChange = vi.fn();
      render(() => (
        <MarkdownEditor
          value="Hello World"
          onChange={onChange}
          ref={(api) => {
            editorApi = api;
          }} />
      ));

      expect(editorApi).toBeDefined();
      expect(typeof editorApi?.insertAtCursor).toBe("function");
    });

    it("insertAtCursor inserts text at cursor position", async () => {
      let currentValue = "Hello World";
      const onChange = (val: string) => {
        currentValue = val;
      };

      render(() => (
        <MarkdownEditor
          value={currentValue}
          onChange={onChange}
          ref={(api) => {
            editorApi = api;
          }} />
      ));

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;

      textarea.focus();
      textarea.setSelectionRange(6, 6);
      editorApi?.insertAtCursor("**", "**");
      expect(currentValue).toBe("Hello ****World");
    });

    it("insertAtCursor wraps selected text", async () => {
      let currentValue = "Hello World";
      const onChange = (val: string) => {
        currentValue = val;
      };

      render(() => (
        <MarkdownEditor
          value={currentValue}
          onChange={onChange}
          ref={(api) => {
            editorApi = api;
          }} />
      ));

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;

      textarea.focus();
      textarea.setSelectionRange(6, 11);
      editorApi?.insertAtCursor("**", "**");
      expect(currentValue).toBe("Hello **World**");
    });

    it("focus method focuses the textarea", () => {
      const onChange = vi.fn();
      render(() => (
        <MarkdownEditor
          value=""
          onChange={onChange}
          ref={(api) => {
            editorApi = api;
          }} />
      ));

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      expect(document.activeElement).not.toBe(textarea);

      editorApi?.focus();
      expect(document.activeElement).toBe(textarea);
    });

    it("getTextarea returns the textarea element", () => {
      const onChange = vi.fn();
      render(() => (
        <MarkdownEditor
          value=""
          onChange={onChange}
          ref={(api) => {
            editorApi = api;
          }} />
      ));

      const textarea = screen.getByRole("textbox");
      const apiTextarea = editorApi?.getTextarea();

      expect(apiTextarea).toBe(textarea);
    });
  });

  describe("Line Numbers", () => {
    it("updates line count when text changes", async () => {
      const [value, setValue] = createSignal("line1");
      render(() => <MarkdownEditor value={value()} onChange={setValue} />);

      expect(screen.getByText("1")).toBeInTheDocument();
      expect(screen.queryByText("2")).not.toBeInTheDocument();

      setValue("line1\nline2");

      expect(screen.getByText("1")).toBeInTheDocument();
      expect(screen.getByText("2")).toBeInTheDocument();
    });

    it("shows at least one line number for empty content", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="" onChange={onChange} />);
      expect(screen.getByText("1")).toBeInTheDocument();
    });
  });

  describe("Font Selection", () => {
    it("applies JetBrains Mono font by default", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="test" onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      expect(textarea.style.fontFamily).toContain("JetBrains Mono");
    });

    it("applies custom font when specified", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="test" onChange={onChange} font="neon" />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      expect(textarea.style.fontFamily).toContain("Monaspace Neon");
    });
  });

  describe("Edge Cases", () => {
    it("Tab key inserts 2-space soft tab", () => {
      let currentValue = "Hello";
      const onChange = (val: string) => {
        currentValue = val;
      };

      render(() => <MarkdownEditor value={currentValue} onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      textarea.focus();
      textarea.setSelectionRange(5, 5);

      fireEvent.keyDown(textarea, { key: "Tab" });

      expect(currentValue).toBe("Hello  ");
    });

    it("Tab key is prevented from default behavior", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="test" onChange={onChange} />);

      const textarea = screen.getByRole("textbox");
      const event = new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true });
      const preventDefaultSpy = vi.spyOn(event, "preventDefault");

      textarea.dispatchEvent(event);

      expect(preventDefaultSpy).toHaveBeenCalled();
    });

    it("paste event triggers onChange with pasted content", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="Hello " onChange={onChange} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
      textarea.focus();
      textarea.setSelectionRange(6, 6);

      fireEvent.input(textarea, { target: { value: "Hello World" } });
      expect(onChange).toHaveBeenCalledWith("Hello World");
    });

    it("handles rapid typing without crashing", async () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="" onChange={onChange} />);

      const textarea = screen.getByRole("textbox");

      for (let i = 0; i < 10; i++) {
        fireEvent.input(textarea, { target: { value: "a".repeat(i + 1) } });
      }

      expect(onChange).toHaveBeenCalledTimes(10);
    });
  });

  describe("Current Line Highlighting", () => {
    it("highlights line 1 by default", () => {
      const [value, setValue] = createSignal("line1\nline2\nline3");
      render(() => <MarkdownEditor value={value()} onChange={setValue} />);

      const line1 = screen.getByText("1");
      expect(line1.className).toContain("text-blue-400");

      const line2 = screen.getByText("2");
      expect(line2.className).not.toContain("text-blue-400");
    });

    it("updates highlight when cursor moves to different line", () => {
      const [value, setValue] = createSignal("line1\nline2\nline3");
      render(() => <MarkdownEditor value={value()} onChange={setValue} />);

      const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;

      textarea.focus();
      textarea.setSelectionRange(6, 6);
      fireEvent.click(textarea);
      const line2 = screen.getByText("2");
      expect(line2.className).toContain("text-blue-400");

      const line1 = screen.getByText("1");
      expect(line1.className).not.toContain("text-blue-400");
    });
  });

  describe("Accessibility", () => {
    it("has aria-label on textarea", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="test" onChange={onChange} />);

      const textarea = screen.getByRole("textbox");
      expect(textarea).toHaveAttribute("aria-label", "Markdown editor");
    });

    it("has aria-multiline on textarea", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="test" onChange={onChange} />);

      const textarea = screen.getByRole("textbox");
      expect(textarea).toHaveAttribute("aria-multiline", "true");
    });

    it("line numbers are aria-hidden", () => {
      const onChange = vi.fn();
      render(() => <MarkdownEditor value="line1\nline2" onChange={onChange} />);

      const lineContainer = screen.getByText("1").parentElement;
      expect(lineContainer).toHaveAttribute("aria-hidden", "true");
    });
  });
});
