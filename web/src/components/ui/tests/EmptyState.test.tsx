import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { EmptyState } from "../EmptyState";

describe("EmptyState", () => {
  afterEach(cleanup);

  it("renders title", () => {
    render(() => <EmptyState title="No items" />);
    expect(screen.getByText("No items")).toBeInTheDocument();
  });

  it("renders description when provided", () => {
    render(() => <EmptyState title="Empty" description="Create your first item" />);
    expect(screen.getByText("Create your first item")).toBeInTheDocument();
  });

  it("renders custom icon when provided", () => {
    render(() => <EmptyState title="Empty" icon={<span data-testid="custom-icon">🎯</span>} />);
    expect(screen.getByTestId("custom-icon")).toBeInTheDocument();
  });

  it("renders action when provided", () => {
    render(() => <EmptyState title="Empty" action={<button>Create</button>} />);
    expect(screen.getByRole("button", { name: "Create" })).toBeInTheDocument();
  });

  it("renders default icon when no custom icon provided", () => {
    render(() => <EmptyState title="Empty" />);
    const svg = document.querySelector("svg");
    expect(svg).toBeInTheDocument();
  });
});
