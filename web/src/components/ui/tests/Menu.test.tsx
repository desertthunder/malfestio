import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Menu, type MenuItem } from "../Menu";

const items: MenuItem[] = [{ id: "edit", label: "Edit", shortcut: "⌘E" }, {
  id: "delete",
  label: "Delete",
  danger: true,
}];

describe("Menu", () => {
  afterEach(cleanup);

  it("renders trigger", () => {
    render(() => <Menu items={items} trigger={<button>Open</button>} />);
    expect(screen.getByText("Open")).toBeInTheDocument();
  });

  it("opens menu on trigger click", () => {
    render(() => <Menu items={items} trigger={<button>Open</button>} />);
    fireEvent.click(screen.getByText("Open"));
    expect(screen.getByRole("menu")).toBeInTheDocument();
    expect(screen.getByText("Edit")).toBeInTheDocument();
  });

  it("calls onClick when item clicked", () => {
    const handleClick = vi.fn();
    const itemsWithHandler: MenuItem[] = [{ id: "action", label: "Action", onClick: handleClick }];
    render(() => <Menu items={itemsWithHandler} trigger={<button>Open</button>} />);
    fireEvent.click(screen.getByText("Open"));
    fireEvent.click(screen.getByText("Action"));
    expect(handleClick).toHaveBeenCalled();
  });

  it("displays keyboard shortcuts", () => {
    render(() => <Menu items={items} trigger={<button>Open</button>} />);
    fireEvent.click(screen.getByText("Open"));
    expect(screen.getByText("⌘E")).toBeInTheDocument();
  });
});
