import { Button } from "$components/ui/Button";
import { Dialog } from "$components/ui/Dialog";
import { api } from "$lib/api";
import type { Persona } from "$lib/model";
import { type Component, createSignal, For } from "solid-js";
import { Motion } from "solid-motionone";

type PersonaOption = { id: Persona; title: string; description: string; icon: string; action: string };

const personas: PersonaOption[] = [{
  id: "learner",
  title: "Learner",
  description: "Study content created by others. Master new topics with spaced repetition.",
  icon: "i-bi-book",
  action: "Browse the Discovery page",
}, {
  id: "creator",
  title: "Creator",
  description: "Build your own decks from articles, lectures, or scratch.",
  icon: "i-bi-pencil",
  action: "Create your first deck",
}, {
  id: "curator",
  title: "Curator",
  description: "Discover, organize, and share the best learning content with others.",
  icon: "i-bi-collection",
  action: "Follow creators in your field",
}];

type Props = { open: boolean; onComplete: (persona: Persona) => void };

export const OnboardingDialog: Component<Props> = (props) => {
  const [selected, setSelected] = createSignal<Persona | null>(null);
  const [submitting, setSubmitting] = createSignal(false);

  const handleConfirm = async () => {
    const persona = selected();
    if (!persona) return;

    setSubmitting(true);
    try {
      await api.updatePreferences({ persona, complete_onboarding: true });
      props.onComplete(persona);
    } catch (e) {
      console.error("Failed to save preferences:", e);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Dialog
      open={props.open}
      onClose={() => {}}
      title="Welcome to Malfestio"
      actions={
        <Button onClick={handleConfirm} disabled={!selected() || submitting()} class="w-full sm:w-auto">
          {submitting() ? "Getting Started..." : "Get Started"}
        </Button>
      }>
      <div class="space-y-6">
        <p class="text-[#C6C6C6] font-light">
          How do you want to use Malfestio? Pick your primary focus — you can always do everything!
        </p>

        <div class="grid gap-3">
          <For each={personas}>
            {(persona, i) => (
              <Motion.button
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.3, delay: i() * 0.1 }}
                onClick={() => setSelected(persona.id)}
                class={`text-left p-4 rounded-lg border transition-all ${
                  selected() === persona.id
                    ? "border-[#0F62FE] bg-[#0F62FE]/10"
                    : "border-[#393939] bg-[#262626] hover:border-[#525252]"
                }`}>
                <div class="flex items-start gap-3">
                  <div class={`text-2xl mt-0.5 ${selected() === persona.id ? "text-[#0F62FE]" : "text-[#8D8D8D]"}`}>
                    <span class={persona.icon} />
                  </div>
                  <div class="flex-1">
                    <div class="flex items-center gap-2 mb-1">
                      <h3 class={`font-medium ${selected() === persona.id ? "text-[#F4F4F4]" : "text-[#C6C6C6]"}`}>
                        {persona.title}
                      </h3>
                      {selected() === persona.id && <span class="i-bi-check-circle-fill text-[#0F62FE] text-sm" />}
                    </div>
                    <p class="text-sm text-[#8D8D8D] mb-2">{persona.description}</p>
                    <p class="text-xs text-[#525252]">
                      <span class="text-[#0F62FE]">→</span> {persona.action}
                    </p>
                  </div>
                </div>
              </Motion.button>
            )}
          </For>
        </div>

        <p class="text-xs text-[#525252] text-center">You can change this anytime in Settings.</p>
      </div>
    </Dialog>
  );
};

export default OnboardingDialog;
