import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, describe, expect, it } from "vitest";
import { Skeleton, SkeletonAvatar, SkeletonText } from "../Skeleton";

describe("Skeleton", () => {
  afterEach(cleanup);

  it("renders basic skeleton", () => {
    render(() => <Skeleton width="100px" height="20px" />);
    const skeleton = document.querySelector("[aria-hidden=\"true\"]");
    expect(skeleton).toBeInTheDocument();
    expect(skeleton).toHaveClass("animate-pulse");
  });

  it("applies rounded classes", () => {
    render(() => <Skeleton rounded="full" />);
    const skeleton = document.querySelector("[aria-hidden=\"true\"]");
    expect(skeleton).toHaveClass("rounded-full");
  });
});

describe("SkeletonText", () => {
  afterEach(cleanup);

  it("renders multiple lines", () => {
    render(() => <SkeletonText lines={4} />);
    const skeletons = document.querySelectorAll("[aria-hidden=\"true\"]");
    expect(skeletons.length).toBe(4);
  });
});

describe("SkeletonAvatar", () => {
  afterEach(cleanup);

  it("renders circular skeleton", () => {
    render(() => <SkeletonAvatar size="lg" />);
    const skeleton = document.querySelector("[aria-hidden=\"true\"]");
    expect(skeleton).toHaveClass("rounded-full");
  });
});
