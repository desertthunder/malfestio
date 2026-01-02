import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorToolbar } from "../EditorToolbar";

describe("EditorToolbar", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockHandlers = {
    onBold: vi.fn(),
    onItalic: vi.fn(),
    onHeading: vi.fn(),
    onLink: vi.fn(),
    onCode: vi.fn(),
    onCodeBlock: vi.fn(),
    onWikilink: vi.fn(),
    onList: vi.fn(),
  };

  it("renders all toolbar buttons", () => {
    render(() => <EditorToolbar {...mockHandlers} />);

    expect(screen.getByTitle(/Bold/)).toBeInTheDocument();
    expect(screen.getByTitle(/Italic/)).toBeInTheDocument();
    expect(screen.getByTitle(/Heading 1/)).toBeInTheDocument();
    expect(screen.getByTitle(/Heading 2/)).toBeInTheDocument();
    expect(screen.getByTitle(/Heading 3/)).toBeInTheDocument();
    expect(screen.getByTitle(/Link/)).toBeInTheDocument();
    expect(screen.getByTitle(/Inline Code/)).toBeInTheDocument();
    expect(screen.getByTitle(/Code Block/)).toBeInTheDocument();
    expect(screen.getByTitle(/Wikilink/)).toBeInTheDocument();
    expect(screen.getByTitle(/Bullet List/)).toBeInTheDocument();
  });

  it("calls onBold when bold button clicked", async () => {
    render(() => <EditorToolbar {...mockHandlers} />);

    const boldBtn = screen.getByTitle(/Bold/);
    boldBtn.click();

    expect(mockHandlers.onBold).toHaveBeenCalledTimes(1);
  });

  it("calls onItalic when italic button clicked", async () => {
    render(() => <EditorToolbar {...mockHandlers} />);

    const italicBtn = screen.getByTitle(/Italic/);
    italicBtn.click();

    expect(mockHandlers.onItalic).toHaveBeenCalledTimes(1);
  });

  it("calls onHeading with level when heading button clicked", async () => {
    render(() => <EditorToolbar {...mockHandlers} />);

    screen.getByTitle(/Heading 1/).click();
    expect(mockHandlers.onHeading).toHaveBeenCalledWith(1);

    screen.getByTitle(/Heading 2/).click();
    expect(mockHandlers.onHeading).toHaveBeenCalledWith(2);

    screen.getByTitle(/Heading 3/).click();
    expect(mockHandlers.onHeading).toHaveBeenCalledWith(3);
  });

  it("calls onLink when link button clicked", async () => {
    render(() => <EditorToolbar {...mockHandlers} />);

    screen.getByTitle(/Link/).click();
    expect(mockHandlers.onLink).toHaveBeenCalledTimes(1);
  });

  it("calls onWikilink when wikilink button clicked", async () => {
    render(() => <EditorToolbar {...mockHandlers} />);

    screen.getByTitle(/Wikilink/).click();
    expect(mockHandlers.onWikilink).toHaveBeenCalledTimes(1);
  });
});
