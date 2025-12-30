import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { type Tab, Tabs } from "../Tabs";

const tabs: Tab[] = [{ id: "tab1", label: "First" }, { id: "tab2", label: "Second" }, {
  id: "tab3",
  label: "Third",
  disabled: true,
}];

describe("Tabs", () => {
  afterEach(cleanup);

  it("renders tab list", () => {
    render(() => <Tabs tabs={tabs} />);
    expect(screen.getByRole("tablist")).toBeInTheDocument();
    expect(screen.getByText("First")).toBeInTheDocument();
    expect(screen.getByText("Second")).toBeInTheDocument();
  });

  it("selects first tab by default", () => {
    render(() => <Tabs tabs={tabs} />);
    const first = screen.getByText("First");
    expect(first).toHaveAttribute("aria-selected", "true");
  });

  it("switches tabs on click", () => {
    const handleChange = vi.fn();
    render(() => <Tabs tabs={tabs} onTabChange={handleChange} />);
    fireEvent.click(screen.getByText("Second"));
    expect(handleChange).toHaveBeenCalledWith("tab2");
  });

  it("respects disabled state", () => {
    const handleChange = vi.fn();
    render(() => <Tabs tabs={tabs} onTabChange={handleChange} />);
    const disabled = screen.getByText("Third");
    fireEvent.click(disabled);
    expect(handleChange).not.toHaveBeenCalled();
  });
});
