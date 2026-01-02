import { DensityProvider } from "$lib/density-context";
import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Button } from "../Button";

describe("Button", () => {
  afterEach(cleanup);

  it("renders with default props", () => {
    render(() => <Button>Click me</Button>);
    const button = screen.getByRole("button", { name: /click me/i });
    expect(button).toBeInTheDocument();
  });

  it("renders with correct variant", () => {
    render(() => <Button variant="secondary">Secondary</Button>);
    const button = screen.getByRole("button", { name: /secondary/i });
    expect(button).toHaveClass("bg-gray-800");
  });

  it("renders with correct size", () => {
    render(() => <Button size="lg">Large</Button>);
    const button = screen.getByRole("button", { name: /large/i });
    expect(button).toHaveClass("px-6 py-3");
  });

  it("handles click events", () => {
    const handleClick = vi.fn();
    render(() => <Button onClick={handleClick}>Click me</Button>);
    const button = screen.getByRole("button", { name: /click me/i });

    fireEvent.click(button);
    expect(handleClick).toHaveBeenCalled();
  });

  describe("density support", () => {
    it("renders with density prop", () => {
      render(() => <Button density="compact">Button</Button>);
      const button = screen.getByRole("button", { name: /button/i });
      expect(button).toBeInTheDocument();
    });

    it("renders within DensityProvider", () => {
      render(() => (
        <DensityProvider>
          <Button>Button</Button>
        </DensityProvider>
      ));
      const button = screen.getByRole("button", { name: /button/i });
      expect(button).toBeInTheDocument();
    });

    it("accepts all density modes", () => {
      const { unmount } = render(() => <Button density="compact">Compact</Button>);
      expect(screen.getByRole("button")).toBeInTheDocument();
      unmount();

      render(() => <Button density="spacious">Spacious</Button>);
      expect(screen.getByRole("button")).toBeInTheDocument();
    });
  });
});
