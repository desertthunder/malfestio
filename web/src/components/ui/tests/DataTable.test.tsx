import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { type Column, DataTable } from "../DataTable";

type TestRow = { id: string; name: string; status: string };

const columns: Column<TestRow>[] = [{ key: "name", header: "Name", sortable: true }, {
  key: "status",
  header: "Status",
}];

const data: TestRow[] = [{ id: "1", name: "Alice", status: "Active" }, { id: "2", name: "Bob", status: "Inactive" }, {
  id: "3",
  name: "Charlie",
  status: "Active",
}];

describe("DataTable", () => {
  afterEach(cleanup);

  it("renders table with data", () => {
    render(() => <DataTable columns={columns} data={data} getRowId={(r) => r.id} />);
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("Alice")).toBeInTheDocument();
    expect(screen.getByText("Bob")).toBeInTheDocument();
  });

  it("sorts by column when clicked", () => {
    render(() => <DataTable columns={columns} data={data} getRowId={(r) => r.id} />);
    const nameHeader = screen.getByText("Name");
    fireEvent.click(nameHeader);
    const rows = screen.getAllByRole("row");
    expect(rows.length).toBe(4);
  });

  it("allows row selection", () => {
    const handleSelection = vi.fn();
    render(() => (
      <DataTable
        columns={columns}
        data={data}
        getRowId={(r) => r.id}
        selectable
        onSelectionChange={handleSelection} />
    ));
    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes.length).toBe(4);
    fireEvent.click(checkboxes[1]);
    expect(handleSelection).toHaveBeenCalled();
  });
});
