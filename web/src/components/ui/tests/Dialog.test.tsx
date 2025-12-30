import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Button } from "../Button";
import { Dialog } from "../Dialog";

describe("Dialog", () => {
  afterEach(cleanup);

  it("renders when open", () => {
    render(() => <Dialog open={true} onClose={() => {}} title="Test Dialog">Dialog content</Dialog>);
    expect(screen.getByRole("dialog")).toBeInTheDocument();
    expect(screen.getByText("Test Dialog")).toBeInTheDocument();
    expect(screen.getByText("Dialog content")).toBeInTheDocument();
  });

  it("does not render when closed", () => {
    render(() => <Dialog open={false} onClose={() => {}} title="Test Dialog">Dialog content</Dialog>);
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("calls onClose when backdrop clicked", () => {
    const handleClose = vi.fn();
    render(() => <Dialog open={true} onClose={handleClose} title="Test Dialog">Content</Dialog>);
    const backdrop = document.querySelector("[aria-hidden=\"true\"]");
    fireEvent.click(backdrop!);
    expect(handleClose).toHaveBeenCalled();
  });

  it("renders actions", () => {
    render(() => (
      <Dialog open={true} onClose={() => {}} title="Confirm" actions={<Button>Confirm</Button>}>Are you sure?</Dialog>
    ));
    expect(screen.getByRole("button", { name: /confirm/i })).toBeInTheDocument();
  });
});
