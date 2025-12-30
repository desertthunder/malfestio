import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { Avatar } from "./Avatar";

describe("Avatar", () => {
  afterEach(cleanup);

  it("renders initials fallback when no src", () => {
    render(() => <Avatar name="John Doe" />);
    expect(screen.getByText("JD")).toBeInTheDocument();
  });

  it("renders image when src provided", () => {
    render(() => <Avatar src="/avatar.jpg" alt="User" />);
    const img = screen.getByRole("img");
    expect(img).toHaveAttribute("src", "/avatar.jpg");
  });

  it("applies size classes", () => {
    render(() => <Avatar name="Test" size="lg" />);
    const avatar = screen.getByText("T").closest("div");
    expect(avatar).toHaveClass("w-12");
  });
});
