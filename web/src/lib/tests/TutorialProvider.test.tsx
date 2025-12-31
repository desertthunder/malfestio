import { render } from "@solidjs/testing-library";
import { describe, expect, it, vi } from "vitest";

vi.mock("$lib/api", () => ({ api: { updatePreferences: vi.fn().mockResolvedValue({ ok: true }) } }));

vi.mock(
  "$lib/store",
  () => ({ prefStore: { prefs: vi.fn(() => ({ tutorial_deck_completed: false })), fetchPrefs: vi.fn() } }),
);

import { DECK_CREATION_STEPS, TutorialProvider, useTutorial } from "../TutorialProvider";

describe("DECK_CREATION_STEPS", () => {
  it("has predefined deck creation steps", () => {
    expect(DECK_CREATION_STEPS.length).toBeGreaterThan(0);
    expect(DECK_CREATION_STEPS[0]).toHaveProperty("id");
    expect(DECK_CREATION_STEPS[0]).toHaveProperty("title");
    expect(DECK_CREATION_STEPS[0]).toHaveProperty("desc");
  });
});

describe("useTutorial", () => {
  it("throws error when used outside provider", () => {
    expect(() => {
      const TestComponent = () => {
        useTutorial();
        return null;
      };
      render(() => <TestComponent />);
    }).toThrow("useTutorial must be used within a TutorialProvider");
  });
});

describe("TutorialProvider", () => {
  it("provides tutorial context to children", () => {
    let tutorial: ReturnType<typeof useTutorial> | undefined;

    const Consumer = () => {
      tutorial = useTutorial();
      return null;
    };

    render(() => (
      <TutorialProvider>
        <Consumer />
      </TutorialProvider>
    ));

    expect(tutorial).toBeDefined();
    expect(tutorial!.active()).toBe(false);
    expect(tutorial!.currentStepIndex()).toBe(0);
  });

  it("starts tutorial when startTutorial is called", () => {
    let tutorial: ReturnType<typeof useTutorial> | undefined;

    const Consumer = () => {
      tutorial = useTutorial();
      return null;
    };

    render(() => (
      <TutorialProvider>
        <Consumer />
      </TutorialProvider>
    ));

    tutorial!.startTutorial();
    expect(tutorial!.active()).toBe(true);
    expect(tutorial!.currentStepIndex()).toBe(0);
  });

  it("advances to next step", () => {
    let tutorial: ReturnType<typeof useTutorial> | undefined;

    const Consumer = () => {
      tutorial = useTutorial();
      return null;
    };

    render(() => (
      <TutorialProvider>
        <Consumer />
      </TutorialProvider>
    ));

    tutorial!.startTutorial();
    tutorial!.nextStep();
    expect(tutorial!.currentStepIndex()).toBe(1);
  });

  it("goes back to previous step", () => {
    let tutorial: ReturnType<typeof useTutorial> | undefined;

    const Consumer = () => {
      tutorial = useTutorial();
      return null;
    };

    render(() => (
      <TutorialProvider>
        <Consumer />
      </TutorialProvider>
    ));

    tutorial!.startTutorial();
    tutorial!.nextStep();
    tutorial!.nextStep();
    tutorial!.prevStep();
    expect(tutorial!.currentStepIndex()).toBe(1);
  });

  it("does not go below step 0", () => {
    let tutorial: ReturnType<typeof useTutorial> | undefined;

    const Consumer = () => {
      tutorial = useTutorial();
      return null;
    };

    render(() => (
      <TutorialProvider>
        <Consumer />
      </TutorialProvider>
    ));

    tutorial!.startTutorial();
    tutorial!.prevStep();
    expect(tutorial!.currentStepIndex()).toBe(0);
  });

  it("calculates progress correctly", () => {
    let tutorial: ReturnType<typeof useTutorial> | undefined;

    const Consumer = () => {
      tutorial = useTutorial();
      return null;
    };

    render(() => (
      <TutorialProvider>
        <Consumer />
      </TutorialProvider>
    ));

    tutorial!.startTutorial();
    const stepCount = tutorial!.steps().length;
    expect(tutorial!.progress()).toBe((1 / stepCount) * 100);

    tutorial!.nextStep();
    expect(tutorial!.progress()).toBe((2 / stepCount) * 100);
  });

  it("registers and unregisters targets", () => {
    let tutorial: ReturnType<typeof useTutorial> | undefined;
    const mockElement = document.createElement("div");

    const Consumer = () => {
      tutorial = useTutorial();
      return null;
    };

    render(() => (
      <TutorialProvider>
        <Consumer />
      </TutorialProvider>
    ));

    tutorial!.registerTarget("test", mockElement);
    expect(tutorial!.targets().get("test")).toBe(mockElement);

    tutorial!.unregisterTarget("test");
    expect(tutorial!.targets().has("test")).toBe(false);
  });
});
