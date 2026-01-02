import { DensityProvider } from "$lib/density-context";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { Card } from "../Card";

describe("Card", () => {
  afterEach(cleanup);

  it("renders children content", () => {
    render(() => <Card>Test Content</Card>);
    expect(screen.getByText("Test Content")).toBeInTheDocument();
  });

  it("renders title when provided", () => {
    render(() => <Card title="Card Title">Content</Card>);
    expect(screen.getByText("Card Title")).toBeInTheDocument();
    expect(screen.getByText("Content")).toBeInTheDocument();
  });

  it("does not render title area if no title provided", () => {
    render(() => <Card>Content</Card>);
    expect(screen.queryByRole("heading")).not.toBeInTheDocument();
  });

  it("accepts density prop", () => {
    render(() => <Card density="compact">Content</Card>);
    expect(screen.getByText("Content")).toBeInTheDocument();
  });

  it("renders with different density modes", () => {
    const { unmount } = render(() => <Card density="spacious">Content</Card>);
    expect(screen.getByText("Content")).toBeInTheDocument();
    unmount();

    render(() => (
      <DensityProvider>
        <Card>Content</Card>
      </DensityProvider>
    ));
    expect(screen.getByText("Content")).toBeInTheDocument();
  });
});
