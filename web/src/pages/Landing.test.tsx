import { MemoryRouter, Route } from "@solidjs/router";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import Landing from "./Landing";

describe("Landing Page", () => {
  afterEach(cleanup);

  function renderLanding() {
    render(() => (
      <MemoryRouter>
        <Route path="/" component={Landing} />
      </MemoryRouter>
    ));
  }
  it("renders hero text correctly", () => {
    renderLanding();

    expect(screen.getByText(/A Learning OS/i)).toBeInTheDocument();
    expect(screen.getByText(/for daily study/i)).toBeInTheDocument();
    expect(screen.getByText(/Master complex topics/i)).toBeInTheDocument();
  });

  it("renders 'Get Started' link pointing to login", () => {
    renderLanding();

    const cta = screen.getByRole("link", { name: /Get Started/i });
    expect(cta).toBeInTheDocument();
    expect(cta).toHaveAttribute("href", "/login");
  });

  it("renders feature grid items", () => {
    renderLanding();

    expect(screen.getByText("Flashcards")).toBeInTheDocument();
    expect(screen.getByText("Linked Notes")).toBeInTheDocument();
    expect(screen.getByText("Social Learning")).toBeInTheDocument();
  });
});
