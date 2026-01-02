import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { DensityProvider, useDensity } from "../density-context";

function TestComponent() {
  const density = useDensity();
  return <div data-testid="density">{density}</div>;
}

describe("density-context", () => {
  afterEach(cleanup);

  it("provides default comfortable density", () => {
    render(() => (
      <DensityProvider>
        <TestComponent />
      </DensityProvider>
    ));

    const element = screen.getByTestId("density");
    expect(element.textContent).toBe("comfortable");
  });

  it("applies density class to container", () => {
    const { container } = render(() => (
      <DensityProvider>
        <TestComponent />
      </DensityProvider>
    ));

    const densityDiv = container.querySelector(".density-comfortable");
    expect(densityDiv).toBeInTheDocument();
  });

  it("useDensity hook provides density accessor", () => {
    render(() => (
      <DensityProvider>
        <div data-testid="test">
          {(() => {
            const density = useDensity();
            return <span>{density}</span>;
          })()}
        </div>
      </DensityProvider>
    ));

    const testDiv = screen.getByTestId("test");
    expect(testDiv.textContent).toBe("comfortable");
  });
});
