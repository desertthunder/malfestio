import { useTutorial } from "$lib/TutorialProvider";
import { Button } from "$ui/Button";
import type { Component } from "solid-js";
import { createEffect, createMemo, createSignal, Index, onCleanup, Show } from "solid-js";
import { Motion, Presence } from "solid-motionone";

type Position = { top: number; left: number; width: number; height: number };

const getTooltipPosition = (target: Position, placement: "top" | "bottom" | "left" | "right") => {
  const padding = 12;
  const tooltipWidth = 320;

  switch (placement) {
    case "top":
      return {
        top: target.top - padding - 8,
        left: target.left + target.width / 2 - tooltipWidth / 2,
        transform: "translateY(-100%)",
      };
    case "bottom":
      return { top: target.top + target.height + padding, left: target.left + target.width / 2 - tooltipWidth / 2 };
    case "left":
      return {
        top: target.top + target.height / 2,
        left: target.left - padding - tooltipWidth,
        transform: "translateY(-50%)",
      };
    case "right":
      return {
        top: target.top + target.height / 2,
        left: target.left + target.width + padding,
        transform: "translateY(-50%)",
      };
    default:
      return { top: target.top + target.height + padding, left: target.left };
  }
};

export const TutorialOverlay: Component = () => {
  const tutorial = useTutorial();
  const [targetPos, setTargetPos] = createSignal<Position | null>(null);

  const currentTarget = createMemo(() => {
    const step = tutorial.currentStep();
    if (!step) return null;
    return tutorial.targets().get(step.id) ?? null;
  });

  createEffect(() => {
    if (!tutorial.active()) return;

    const element = currentTarget();
    if (!element) {
      setTargetPos(null);
      return;
    }

    const updatePosition = () => {
      const rect = element.getBoundingClientRect();
      setTargetPos({
        top: rect.top + window.scrollY,
        left: rect.left + window.scrollX,
        width: rect.width,
        height: rect.height,
      });
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition);
    onCleanup(() => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition);
    });
  });

  createEffect(() => {
    if (!tutorial.active()) return;
    const element = currentTarget();
    if (element && typeof element.scrollIntoView === "function") {
      element.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  });

  return (
    <Presence>
      <Show when={tutorial.active()}>
        <Motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.2 }}
          class="fixed inset-0 z-50 pointer-events-none">
          <Show when={targetPos()}>
            {(pos) => (
              <>
                <svg
                  class="absolute inset-0 w-full h-full pointer-events-auto"
                  style={{ height: `${document.documentElement.scrollHeight}px` }}>
                  <defs>
                    <mask id="spotlight-mask">
                      <rect width="100%" height="100%" fill="white" />
                      <rect
                        x={pos().left - 8}
                        y={pos().top - 8}
                        width={pos().width + 16}
                        height={pos().height + 16}
                        rx="8"
                        fill="black" />
                    </mask>
                  </defs>
                  <rect
                    width="100%"
                    height="100%"
                    fill="rgba(0,0,0,0.75)"
                    mask="url(#spotlight-mask)"
                    onClick={() => tutorial.skipTutorial()} />
                </svg>

                <div
                  class="absolute border-2 border-[#0F62FE] rounded-lg pointer-events-none"
                  style={{
                    top: `${pos().top - 8}px`,
                    left: `${pos().left - 8}px`,
                    width: `${pos().width + 16}px`,
                    height: `${pos().height + 16}px`,
                    "box-shadow": "0 0 0 4px rgba(15, 98, 254, 0.3)",
                  }} />

                <Motion.div
                  initial={{ opacity: 0, scale: 0.95 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ duration: 0.2 }}
                  class="absolute w-80 bg-[#262626] border border-[#393939] rounded-lg shadow-xl p-4 pointer-events-auto"
                  style={{
                    top: `${getTooltipPosition(pos(), tutorial.currentStep()!.placement).top}px`,
                    left: `${getTooltipPosition(pos(), tutorial.currentStep()!.placement).left}px`,
                    transform: getTooltipPosition(pos(), tutorial.currentStep()!.placement).transform,
                  }}>
                  <div class="h-1 bg-[#393939] rounded-full mb-4 overflow-hidden">
                    <div
                      class="h-full bg-[#0F62FE] transition-all duration-300"
                      style={{ width: `${tutorial.progress()}%` }} />
                  </div>

                  <h3 class="text-lg font-medium text-[#F4F4F4] mb-2">{tutorial.currentStep()?.title}</h3>
                  <p class="text-sm text-[#C6C6C6] mb-4">{tutorial.currentStep()?.desc}</p>

                  <div class="flex items-center justify-between">
                    <button
                      onClick={() => tutorial.skipTutorial()}
                      class="text-sm text-[#8D8D8D] hover:text-[#C6C6C6] transition-colors">
                      Skip tutorial
                    </button>
                    <div class="flex gap-2">
                      <Show when={!tutorial.isFirstStep()}>
                        <Button
                          variant="secondary"
                          size="sm"
                          onClick={() => tutorial.prevStep()}>
                          Back
                        </Button>
                      </Show>
                      <Button size="sm" onClick={() => tutorial.nextStep()}>
                        {tutorial.isLastStep() ? "Finish" : "Next"}
                      </Button>
                    </div>
                  </div>

                  <div class="flex justify-center gap-1.5 mt-4">
                    <Index each={tutorial.steps()}>
                      {(_, i) => (
                        <div
                          class={`w-2 h-2 rounded-full transition-colors ${
                            i === tutorial.currentStepIndex() ? "bg-[#0F62FE]" : "bg-[#525252]"
                          }`} />
                      )}
                    </Index>
                  </div>
                  <p class="text-xs text-[#525252] text-center mt-3">Use ← → arrow keys or Esc to skip</p>
                </Motion.div>
              </>
            )}
          </Show>
        </Motion.div>
      </Show>
    </Presence>
  );
};

export default TutorialOverlay;
