import { MemoryRouter, Route } from "@solidjs/router";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { Footer } from "./Footer";

describe("Footer", () => {
  afterEach(cleanup);

  function renderFooter() {
    render(() => (
      <MemoryRouter>
        <Route path="/" component={Footer} />
      </MemoryRouter>
    ));
  }

  it("renders copyright text", () => {
    renderFooter();
    expect(screen.getByText(/© 2025 Stormlight Labs/)).toBeInTheDocument();
  });

  it("renders About link", () => {
    renderFooter();
    const aboutLink = screen.getByRole("link", { name: "About" });
    expect(aboutLink).toHaveAttribute("href", "/about");
  });

  it("renders Tangled link", () => {
    renderFooter();
    const tangledLink = screen.getByRole("link", { name: "Tangled" });
    expect(tangledLink).toHaveAttribute("href", "https://tangled.org/desertthunder.dev/malfestio");
  });

  it("renders GitHub link", () => {
    renderFooter();
    const githubLink = screen.getByRole("link", { name: "GitHub" });
    expect(githubLink).toHaveAttribute("href", "https://github.com/stormlightlabs");
  });
});
