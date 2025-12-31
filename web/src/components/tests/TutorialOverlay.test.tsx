import { TutorialProvider, useTutorial } from "$lib/TutorialProvider";
import { cleanup, render, screen } from "@solidjs/testing-library";
import { createEffect } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import { TutorialOverlay } from "../TutorialOverlay";

vi.mock("$lib/api", () => ({ api: { updatePreferences: vi.fn().mockResolvedValue({ ok: true }) } }));

vi.mock(
  "$lib/store",
  () => ({ prefStore: { prefs: vi.fn(() => ({ tutorial_deck_completed: false })), fetchPrefs: vi.fn() } }),
);

const TutorialTestHarness = (props: { onTutorial?: (t: ReturnType<typeof useTutorial>) => void }) => {
  const tutorial = useTutorial();

  createEffect(() => {
    props.onTutorial?.(tutorial);
  });

  return (
    <>
      <div
        ref={(el) => tutorial.registerTarget("title", el)}
        style={{ position: "absolute", top: "100px", left: "100px", width: "200px", height: "50px" }}>
        Target Element
      </div>
      <TutorialOverlay />
    </>
  );
};

describe("TutorialOverlay", () => {
  afterEach(cleanup);

  it("does not render when tutorial is inactive", () => {
    render(() => (
      <TutorialProvider>
        <TutorialTestHarness />
      </TutorialProvider>
    ));
    expect(screen.queryByText("Name Your Deck")).not.toBeInTheDocument();
  });

  it("renders when tutorial is active and target exists", () => {
    let tutorialRef: ReturnType<typeof useTutorial> | undefined;

    render(() => (
      <TutorialProvider>
        <TutorialTestHarness
          onTutorial={(t) => {
            tutorialRef = t;
          }} />
      </TutorialProvider>
    ));

    tutorialRef!.startTutorial();

    expect(screen.getByText("Name Your Deck")).toBeInTheDocument();
  });

  it("shows skip button when active", () => {
    let tutorialRef: ReturnType<typeof useTutorial> | undefined;

    render(() => (
      <TutorialProvider>
        <TutorialTestHarness
          onTutorial={(t) => {
            tutorialRef = t;
          }} />
      </TutorialProvider>
    ));

    tutorialRef!.startTutorial();

    expect(screen.getByText("Skip tutorial")).toBeInTheDocument();
  });

  it("shows Next button when not on last step", () => {
    let tutorialRef: ReturnType<typeof useTutorial> | undefined;

    render(() => (
      <TutorialProvider>
        <TutorialTestHarness
          onTutorial={(t) => {
            tutorialRef = t;
          }} />
      </TutorialProvider>
    ));

    tutorialRef!.startTutorial();

    expect(screen.getByRole("button", { name: /Next/i })).toBeInTheDocument();
  });

  it("shows keyboard hint", () => {
    let tutorialRef: ReturnType<typeof useTutorial> | undefined;

    render(() => (
      <TutorialProvider>
        <TutorialTestHarness
          onTutorial={(t) => {
            tutorialRef = t;
          }} />
      </TutorialProvider>
    ));

    tutorialRef!.startTutorial();

    expect(screen.getByText(/arrow keys/i)).toBeInTheDocument();
  });
});
