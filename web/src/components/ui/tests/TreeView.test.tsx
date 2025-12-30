import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { type TreeNode, TreeView } from "../TreeView";

const sampleNodes: TreeNode[] = [{
  id: "1",
  label: "Root",
  children: [{ id: "1-1", label: "Child 1" }, { id: "1-2", label: "Child 2" }],
}, { id: "2", label: "Sibling" }];

describe("TreeView", () => {
  afterEach(cleanup);

  it("renders nodes", () => {
    render(() => <TreeView nodes={sampleNodes} />);
    expect(screen.getByRole("tree")).toBeInTheDocument();
    expect(screen.getByText("Root")).toBeInTheDocument();
    expect(screen.getByText("Sibling")).toBeInTheDocument();
  });

  it("expands children on click", () => {
    render(() => <TreeView nodes={sampleNodes} />);
    expect(screen.queryByText("Child 1")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("Root"));
    expect(screen.getByText("Child 1")).toBeInTheDocument();
    expect(screen.getByText("Child 2")).toBeInTheDocument();
  });

  it("calls onSelect when node clicked", () => {
    const handleSelect = vi.fn();
    render(() => <TreeView nodes={sampleNodes} onSelect={handleSelect} />);
    fireEvent.click(screen.getByText("Sibling"));
    expect(handleSelect).toHaveBeenCalledWith(expect.objectContaining({ id: "2" }));
  });
});
