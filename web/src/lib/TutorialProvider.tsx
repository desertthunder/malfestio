import { api } from "$lib/api";
import { prefStore } from "$lib/store";
import type { Accessor, JSX } from "solid-js";
import { createContext, createEffect, createSignal, onCleanup, useContext } from "solid-js";

type TutorialPlacement = "top" | "bottom" | "left" | "right";

export type TutorialStep = { id: string; title: string; desc: string; placement: TutorialPlacement };

export const DECK_CREATION_STEPS: TutorialStep[] = [{
  id: "title",
  title: "Name Your Deck",
  desc: "Start by giving your deck a memorable title. This helps you find it later!",
  placement: "bottom",
}, {
  id: "description",
  title: "Add a Description",
  desc: "Describe what this deck covers. Great for sharing with others!",
  placement: "bottom",
}, {
  id: "tags",
  title: "Add Tags",
  desc: "Tags help categorize your deck. Use topics like 'spanish', 'vocabulary', 'history'.",
  placement: "bottom",
}, {
  id: "visibility",
  title: "Set Visibility",
  desc: "Keep it Private for yourself, or make it Public to share with the community.",
  placement: "bottom",
}, {
  id: "add-card",
  title: "Add Your First Card",
  desc: "Click here to create a flashcard. Add a question on the front and answer on the back!",
  placement: "top",
}];

type TutorialContextValue = {
  active: Accessor<boolean>;
  currentStep: Accessor<TutorialStep | undefined>;
  currentStepIndex: Accessor<number>;
  steps: Accessor<TutorialStep[]>;
  targets: Accessor<Map<string, HTMLElement>>;
  isFirstStep: Accessor<boolean>;
  isLastStep: Accessor<boolean>;
  progress: Accessor<number>;
  shouldShowTutorial: () => boolean;
  startTutorial: () => void;
  nextStep: () => void;
  prevStep: () => void;
  skipTutorial: () => void;
  completeTutorial: () => Promise<void>;
  registerTarget: (id: string, element: HTMLElement) => void;
  unregisterTarget: (id: string) => void;
};

const TutorialContext = createContext<TutorialContextValue>();

export function TutorialProvider(props: { children: JSX.Element; steps?: TutorialStep[] }) {
  const [active, setActive] = createSignal(false);
  const [currentStepIndex, setCurrentStepIndex] = createSignal(0);
  const [targets, setTargets] = createSignal<Map<string, HTMLElement>>(new Map());
  const steps = () => props.steps ?? DECK_CREATION_STEPS;

  const shouldShowTutorial = () => {
    const prefs = prefStore.prefs();
    return prefs !== null && !prefs.tutorial_deck_completed;
  };

  const registerTarget = (id: string, element: HTMLElement) => {
    setTargets((prev) => {
      const next = new Map(prev);
      next.set(id, element);
      return next;
    });
  };

  const unregisterTarget = (id: string) => {
    setTargets((prev) => {
      const next = new Map(prev);
      next.delete(id);
      return next;
    });
  };

  const startTutorial = () => {
    setCurrentStepIndex(0);
    setActive(true);
  };

  const nextStep = () => {
    if (currentStepIndex() < steps().length - 1) {
      setCurrentStepIndex(currentStepIndex() + 1);
    } else {
      completeTutorial();
    }
  };

  const prevStep = () => {
    if (currentStepIndex() > 0) {
      setCurrentStepIndex(currentStepIndex() - 1);
    }
  };

  const skipTutorial = () => completeTutorial();

  const completeTutorial = async () => {
    setActive(false);
    try {
      await api.updatePreferences({ tutorial_deck_completed: true });
      prefStore.fetchPrefs();
    } catch (e) {
      console.error("Failed to mark tutorial complete:", e);
    }
  };

  const currentStep = () => steps()[currentStepIndex()];
  const isFirstStep = () => currentStepIndex() === 0;
  const isLastStep = () => currentStepIndex() === steps().length - 1;
  const progress = () => ((currentStepIndex() + 1) / steps().length) * 100;

  // Keyboard navigation
  createEffect(() => {
    if (!active()) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") skipTutorial();
      if (e.key === "ArrowRight" || e.key === "Enter") nextStep();
      if (e.key === "ArrowLeft") prevStep();
    };

    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  const value: TutorialContextValue = {
    active,
    currentStep,
    currentStepIndex,
    steps,
    targets,
    isFirstStep,
    isLastStep,
    progress,
    shouldShowTutorial,
    startTutorial,
    nextStep,
    prevStep,
    skipTutorial,
    completeTutorial,
    registerTarget,
    unregisterTarget,
  };

  return <TutorialContext.Provider value={value}>{props.children}</TutorialContext.Provider>;
}

export function useTutorial() {
  const context = useContext(TutorialContext);
  if (!context) {
    throw new Error("useTutorial must be used within a TutorialProvider");
  }
  return context;
}

/**
 * Register an element as a tutorial target. Use as a ref directive.
 */
export function useTutorialTarget(stepId: string) {
  const context = useContext(TutorialContext);

  return (element: HTMLElement) => {
    if (!context) return;
    context.registerTarget(stepId, element);
    onCleanup(() => context.unregisterTarget(stepId));
  };
}

export type { TutorialContextValue };
