import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { UserProfileCard } from "../UserProfileCard";

describe("UserProfileCard", () => {
  afterEach(() => cleanup());

  const mockProfile = {
    did: "did:plc:abcdef123456",
    follower_count: 101,
    following_count: 202,
    deck_count: 303,
    indexed_deck_count: 404,
  };

  it("renders user information correctly", () => {
    render(() => <UserProfileCard profile={mockProfile} />);
    expect(screen.getByText("did:plc:abcdef123456")).toBeInTheDocument();
    expect(screen.getByText("AT Protocol User")).toBeInTheDocument();
  });

  it("renders statistics correctly", () => {
    render(() => <UserProfileCard profile={mockProfile} />);
    expect(screen.getByText("101")).toBeInTheDocument();
    expect(screen.getByText("202")).toBeInTheDocument();
    expect(screen.getByText("707")).toBeInTheDocument();
  });
});
