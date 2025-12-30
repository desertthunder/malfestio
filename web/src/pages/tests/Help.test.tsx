import { cleanup, fireEvent, render, screen } from "@solidjs/testing-library";
import { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import Help from "../Help";

vi.mock(
  "@solidjs/router",
  () => ({
    A: (props: { href: string; children: JSX.Element; class?: string }) => (
      <a href={props.href} class={props.class}>{props.children}</a>
    ),
  }),
);

vi.mock("$components/layout/Footer", () => ({ Footer: () => <footer data-testid="footer">Footer</footer> }));

describe("Help Page", () => {
  afterEach(cleanup);

  it("renders the help page header", () => {
    render(() => <Help />);
    expect(screen.getByText("Help Center")).toBeInTheDocument();
  });

  it("displays beta notice", () => {
    render(() => <Help />);
    expect(screen.getByText("Beta Notice:")).toBeInTheDocument();
    expect(screen.getByText(/Malfestio is still in active development/i)).toBeInTheDocument();
  });

  it("shows all FAQ categories", () => {
    render(() => <Help />);
    expect(screen.getByText("Getting Started")).toBeInTheDocument();
    expect(screen.getByText("Spaced Repetition")).toBeInTheDocument();
    expect(screen.getByText("AT Protocol & Privacy")).toBeInTheDocument();
    expect(screen.getByText("Community & Sharing")).toBeInTheDocument();
  });

  it("displays FAQ questions", () => {
    render(() => <Help />);
    expect(screen.getByText("What is Malfestio?")).toBeInTheDocument();
    expect(screen.getByText("What is spaced repetition?")).toBeInTheDocument();
    expect(screen.getByText("What is the AT Protocol?")).toBeInTheDocument();
    expect(screen.getByText("What does 'Fork' mean?")).toBeInTheDocument();
  });

  it("expands accordion when question is clicked", async () => {
    render(() => <Help />);

    expect(screen.queryByText(/Malfestio is a decentralized learning platform/i)).not.toBeInTheDocument();

    const question = screen.getByText("What is Malfestio?");
    fireEvent.click(question);
    expect(screen.getByText(/Malfestio is a decentralized learning platform/i)).toBeInTheDocument();
  });

  it("collapses accordion when clicked again", async () => {
    render(() => <Help />);

    const question = screen.getByText("What is Malfestio?");

    fireEvent.click(question);
    expect(screen.getByText(/Malfestio is a decentralized learning platform/i)).toBeInTheDocument();

    fireEvent.click(question);
    expect(screen.queryByText(/Malfestio is a decentralized learning platform/i)).not.toBeInTheDocument();
  });

  it("has link back to app", () => {
    render(() => <Help />);
    const backLink = screen.getByRole("link", { name: /Back to App/i });
    expect(backLink).toHaveAttribute("href", "/");
  });

  it("shows contact section", () => {
    render(() => <Help />);
    expect(screen.getByText("Still have questions?")).toBeInTheDocument();
    expect(screen.getByText("Bluesky")).toBeInTheDocument();
    expect(screen.getByText("GitHub")).toBeInTheDocument();
  });

  it("includes footer", () => {
    render(() => <Help />);
    expect(screen.getByTestId("footer")).toBeInTheDocument();
  });
});
