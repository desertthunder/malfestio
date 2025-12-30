import { MemoryRouter, Route } from "@solidjs/router";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import About from "../About";

describe("About Page", () => {
  afterEach(cleanup);

  function renderAbout() {
    render(() => (
      <MemoryRouter>
        <Route path="/" component={About} />
      </MemoryRouter>
    ));
  }

  it("renders page title", () => {
    renderAbout();
    expect(screen.getByText("About Malfestio")).toBeInTheDocument();
  });

  it("renders mission statement", () => {
    renderAbout();
    expect(screen.getByText(/decentralized learning platform/i)).toBeInTheDocument();
  });

  it("renders team section with Owais", () => {
    renderAbout();
    expect(screen.getByText("Team")).toBeInTheDocument();
    expect(screen.getByText("Owais")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "desertthunder.dev" })).toHaveAttribute(
      "href",
      "https://desertthunder.dev",
    );
  });

  it("renders links section", () => {
    renderAbout();
    expect(screen.getByText("Links")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /Tangled Repository/i })).toHaveAttribute(
      "href",
      "https://tangled.org/desertthunder.dev/malfestio",
    );
  });
});
