import type { Heading } from "$lib/wikilink";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OutlinePanel } from "../OutlinePanel";

vi.mock(
  "@solidjs/router",
  () => ({
    A: (props: { href: string; children: JSX.Element; onClick?: (e: MouseEvent) => void }) => (
      <a
        href={props.href}
        onClick={(e) => props.onClick?.(e)}>
        {props.children}
      </a>
    ),
  }),
);

describe("OutlinePanel", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  const mockHeadings: Heading[] = [
    { level: 1, text: "Introduction", id: "introduction" },
    { level: 2, text: "Background", id: "background" },
    { level: 2, text: "Methods", id: "methods" },
    { level: 3, text: "Sub Methods", id: "sub-methods" },
    { level: 1, text: "Conclusion", id: "conclusion" },
  ];

  it("renders heading title", () => {
    render(() => <OutlinePanel headings={[]} />);
    expect(screen.getByText("Outline")).toBeInTheDocument();
  });

  it("shows empty state when no headings", () => {
    render(() => <OutlinePanel headings={[]} />);
    expect(screen.getByText("No headings found")).toBeInTheDocument();
  });

  it("renders all headings", () => {
    render(() => <OutlinePanel headings={mockHeadings} />);
    expect(screen.getByText("Introduction")).toBeInTheDocument();
    expect(screen.getByText("Background")).toBeInTheDocument();
    expect(screen.getByText("Methods")).toBeInTheDocument();
    expect(screen.getByText("Sub Methods")).toBeInTheDocument();
    expect(screen.getByText("Conclusion")).toBeInTheDocument();
  });

  it("renders headings as links", () => {
    render(() => <OutlinePanel headings={mockHeadings} />);
    const introLink = screen.getByRole("link", { name: "Introduction" });
    expect(introLink).toHaveAttribute("href", "#introduction");
  });

  it("calls onHeadingClick when heading clicked", () => {
    const onClick = vi.fn();
    render(() => <OutlinePanel headings={mockHeadings} onHeadingClick={onClick} />);
    const introLink = screen.getByRole("link", { name: "Introduction" });
    introLink.click();
    expect(onClick).toHaveBeenCalledWith("introduction");
  });
});
