import { Footer } from "$components/layout/Footer";
import { Button } from "$components/ui/Button";
import { A } from "@solidjs/router";
import type { Component, JSX } from "solid-js";
import { createSignal, For, Show } from "solid-js";
import { Motion } from "solid-motionone";

type FAQItem = { question: string; answer: JSX.Element | string };

type FAQSection = { title: string; icon: string; items: FAQItem[] };

const faqSections: FAQSection[] = [{
  title: "Getting Started",
  icon: "i-bi-rocket-takeoff",
  items: [{
    question: "What is Malfestio?",
    answer:
      "Malfestio is a decentralized learning platform that combines flashcards with spaced repetition. Built on the AT Protocol, your content is portable and you maintain ownership of your data.",
  }, {
    question: "How do I create my first deck?",
    answer:
      "Click 'Create Deck' in your Library. Add a title, description, and tags, then add cards with questions and answers. You can also import content from articles or lectures.",
  }, {
    question: "What makes Malfestio different from other flashcard apps?",
    answer:
      "Malfestio is built on the AT Protocol, meaning your content is decentralized and portable. You can fork and remix others' decks, follow creators, and participate in a community-driven learning ecosystem.",
  }],
}, {
  title: "Spaced Repetition",
  icon: "i-bi-arrow-repeat",
  items: [{
    question: "What is spaced repetition?",
    answer:
      "Spaced repetition is a learning technique that schedules reviews at optimal intervals. Cards you struggle with appear more often; cards you know well appear less frequently.",
  }, {
    question: "How does the grading system work?",
    answer: (
      <ul class="list-disc list-inside space-y-1">
        <li>
          <strong>1 (Again)</strong>: Completely forgot — will review soon
        </li>
        <li>
          <strong>2 (Hard)</strong>: Struggled to remember
        </li>
        <li>
          <strong>3 (Good)</strong>: Remembered with some effort
        </li>
        <li>
          <strong>4 (Easy)</strong>: Remembered easily
        </li>
        <li>
          <strong>5 (Perfect)</strong>: Instant recall
        </li>
      </ul>
    ),
  }, {
    question: "How are review intervals calculated?",
    answer:
      "We use the SM-2 algorithm, a proven spaced repetition method. Intervals grow exponentially for cards you know well, typically starting at 1 day and growing to weeks or months.",
  }],
}, {
  title: "AT Protocol & Privacy",
  icon: "i-bi-globe",
  items: [{
    question: "What is the AT Protocol?",
    answer:
      "The AT Protocol (Authenticated Transfer Protocol) is an open, decentralized social networking protocol. It powers Bluesky and enables portable, user-owned data.",
  }, {
    question: "Is my study data private?",
    answer:
      "Yes! Your review history, grades, and learning progress are stored locally and never published to the network. Only content you explicitly choose to publish (decks, cards) becomes public.",
  }, {
    question: "Can I use my existing Bluesky account?",
    answer:
      "Yes! You can log in with your Bluesky handle and app password. Your decks can be published to your AT Protocol repository.",
  }],
}, {
  title: "Community & Sharing",
  icon: "i-bi-people",
  items: [{
    question: "What does 'Fork' mean?",
    answer:
      "Forking creates a personal copy of someone else's deck. You can study, edit, and improve it. The original deck remains unchanged.",
  }, {
    question: "How do I discover new decks?",
    answer:
      "Use the Discovery page to browse trending decks and popular tags. You can also follow creators and see their latest decks in your feed.",
  }, {
    question: "Can I make my decks private?",
    answer:
      "Yes! Each deck has visibility settings: Private (only you), Unlisted (anyone with link), Public (discoverable by all), or Shared With (specific users).",
  }],
}];

const AccordionItem: Component<{ item: FAQItem; index: number }> = (props) => {
  const [open, setOpen] = createSignal(false);

  return (
    <Motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2, delay: props.index * 0.05 }}
      class="border-b border-[#393939] last:border-b-0">
      <button onClick={() => setOpen(!open())} class="w-full py-4 flex items-center justify-between text-left group">
        <span class="text-[#F4F4F4] group-hover:text-[#0F62FE] transition-colors font-medium">
          {props.item.question}
        </span>
        <span class={`i-bi-chevron-down text-[#8D8D8D] transition-transform ${open() ? "rotate-180" : ""}`} />
      </button>
      <Show when={open()}>
        <Motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          transition={{ duration: 0.2 }}
          class="pb-4 text-[#C6C6C6] font-light leading-relaxed">
          {props.item.answer}
        </Motion.div>
      </Show>
    </Motion.div>
  );
};

const Help: Component = () => {
  return (
    <div class="min-h-screen bg-[#161616] flex flex-col">
      <header class="border-b border-[#262626] bg-[#161616]/95 backdrop-blur sticky top-0 z-50">
        <div class="max-w-4xl mx-auto px-6 py-4 flex items-center justify-between">
          <A href="/" class="text-xl font-medium text-[#F4F4F4] hover:text-[#0F62FE] transition-colors">Malfestio</A>
          <A href="/">
            <Button variant="secondary" size="sm">Back to App</Button>
          </A>
        </div>
      </header>

      <div class="bg-[#0F62FE]/10 border-b border-[#0F62FE]/30">
        <div class="max-w-4xl mx-auto px-6 py-3 flex items-center gap-3">
          <span class="i-bi-info-circle text-[#0F62FE]" />
          <p class="text-sm text-[#C6C6C6]">
            <strong class="text-[#F4F4F4]">Beta Notice:</strong>{" "}
            Malfestio is still in active development. Features may change and some functionality may be incomplete.
          </p>
        </div>
      </div>

      <main class="flex-1 max-w-4xl mx-auto px-6 py-12 w-full">
        <Motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4 }}>
          <h1 class="text-4xl font-light text-[#F4F4F4] mb-2">Help Center</h1>
          <p class="text-[#C6C6C6] mb-12 font-light">Find answers to common questions about using Malfestio.</p>
        </Motion.div>

        <div class="space-y-12">
          <For each={faqSections}>
            {(section, sectionIndex) => (
              <Motion.section
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4, delay: sectionIndex() * 0.1 }}>
                <div class="flex items-center gap-3 mb-6">
                  <span class={`${section.icon} text-2xl text-[#0F62FE]`} />
                  <h2 class="text-xl font-medium text-[#F4F4F4]">{section.title}</h2>
                </div>
                <div class="bg-[#1E1E1E] rounded-lg border border-[#262626] px-6">
                  <For each={section.items}>{(item, i) => <AccordionItem item={item} index={i()} />}</For>
                </div>
              </Motion.section>
            )}
          </For>
        </div>

        <Motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.4, delay: 0.5 }}
          class="mt-16 text-center py-12 border-t border-[#262626]">
          <h3 class="text-lg font-medium text-[#F4F4F4] mb-2">Still have questions?</h3>
          <p class="text-[#C6C6C6] font-light mb-6">We're here to help. Reach out on Bluesky or check our GitHub.</p>
          <div class="flex gap-4 justify-center">
            <a
              href="https://bsky.app"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-2 text-[#0F62FE] hover:underline">
              <span class="i-bi-chat" /> Bluesky
            </a>
            <a
              href="https://github.com"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center gap-2 text-[#0F62FE] hover:underline">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 25 25" class="inline">
                <path
                  fill="currentColor"
                  d="M12.5.75C6.146.75 1 5.896 1 12.25c0 5.089 3.292 9.387 7.863 10.91.575.101.79-.244.79-.546 0-.273-.014-1.178-.014-2.142-2.889.532-3.636-.705-3.866-1.35-.13-.331-.69-1.352-1.18-1.625-.402-.216-.977-.748-.014-.762.906-.014 1.553.834 1.769 1.179 1.035 1.74 2.688 1.25 3.349.948.1-.747.402-1.25.733-1.538-2.559-.287-5.232-1.279-5.232-5.678 0-1.25.446-2.285 1.18-3.09-.115-.288-.517-1.467.115-3.048 0 0 .963-.302 3.163 1.179.92-.259 1.897-.388 2.875-.388.977 0 1.955.13 2.875.388 2.2-1.495 3.162-1.179 3.162-1.179.633 1.581.23 2.76.115 3.048.733.805 1.179 1.825 1.179 3.09 0 4.413-2.688 5.39-5.247 5.678.417.36.776 1.05.776 2.128 0 1.538-.014 2.774-.014 3.162 0 .302.216.662.79.547C20.709 21.637 24 17.324 24 12.25 24 5.896 18.854.75 12.5.75Z" />
              </svg>
              GitHub
            </a>
          </div>
        </Motion.div>
      </main>

      <Footer />
    </div>
  );
};

export default Help;
