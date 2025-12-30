import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Dropdown, type DropdownOption } from "./Dropdown";

const options: DropdownOption[] = [{ value: "a", label: "Option A" }, { value: "b", label: "Option B" }, {
  value: "c",
  label: "Option C",
  disabled: true,
}];

describe("Dropdown", () => {
  afterEach(cleanup);

  it("renders with placeholder", () => {
    render(() => <Dropdown options={options} placeholder="Select..." />);
    expect(screen.getByText("Select...")).toBeInTheDocument();
  });

  it("opens dropdown on click", () => {
    render(() => <Dropdown options={options} />);
    fireEvent.click(screen.getByRole("button"));
    expect(screen.getByRole("listbox")).toBeInTheDocument();
    expect(screen.getByText("Option A")).toBeInTheDocument();
  });

  it("calls onChange when option selected", () => {
    const handleChange = vi.fn();
    render(() => <Dropdown options={options} onChange={handleChange} />);
    fireEvent.click(screen.getByRole("button"));
    fireEvent.click(screen.getByText("Option A"));
    expect(handleChange).toHaveBeenCalledWith("a");
  });

  it("supports multi-select", () => {
    const handleChange = vi.fn();
    render(() => <Dropdown options={options} multiple onChange={handleChange} />);
    fireEvent.click(screen.getByRole("button"));
    fireEvent.click(screen.getByText("Option A"));
    expect(handleChange).toHaveBeenCalledWith(["a"]);
  });
});
