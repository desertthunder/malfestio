import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Tag } from "./Tag";

describe("Tag", () => {
  afterEach(cleanup);

  it("renders label", () => {
    render(() => <Tag label="Test Tag" />);
    expect(screen.getByText("Test Tag")).toBeInTheDocument();
  });

  it("shows dismiss button for dismissible type", () => {
    const handleDismiss = vi.fn();
    render(() => <Tag label="Dismissible" type="dismissible" onDismiss={handleDismiss} />);
    const dismissBtn = screen.getByRole("button", { name: /remove/i });
    expect(dismissBtn).toBeInTheDocument();
    fireEvent.click(dismissBtn);
    expect(handleDismiss).toHaveBeenCalled();
  });

  it("supports selectable type", () => {
    const handleSelect = vi.fn();
    render(() => <Tag label="Selectable" type="selectable" onSelect={handleSelect} />);
    const tag = screen.getByRole("button");
    fireEvent.click(tag);
    expect(handleSelect).toHaveBeenCalled();
  });

  it("shows selected state", () => {
    render(() => <Tag label="Selected" type="selectable" selected color="blue" />);
    const tag = screen.getByText("Selected").parentElement;
    expect(tag).toHaveClass("bg-blue-600");
  });
});
