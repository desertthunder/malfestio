import { A } from "@solidjs/router";
import type { Component } from "solid-js";

const Feature: Component<{ title: string; desc: string }> = (props) => (
  <div class="border border-neutral-800 p-6 hover:border-blue-600 transition-colors group h-full">
    <h3 class="text-xl font-light text-white mb-2 group-hover:text-blue-500 transition-colors">{props.title}</h3>
    <p class="text-neutral-400 font-light leading-relaxed">{props.desc}</p>
  </div>
);

const Landing: Component = () => {
  return (
    <div class="min-h-screen bg-black text-white font-sans selection:bg-blue-500/30">
      <header class="border-b border-neutral-900">
        <div class="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
          <div class="font-bold tracking-tight text-xl">Malfestio</div>
          <A href="/login" class="text-sm font-medium text-neutral-400 hover:text-white transition-colors">Log in</A>
        </div>
      </header>

      <main>
        <section class="max-w-7xl mx-auto px-6 py-24 md:py-32 border-b border-neutral-900">
          <div class="max-w-3xl">
            <h1 class="text-5xl md:text-7xl font-light tracking-tight mb-8 leading-[1.1]">
              Learning on <br />
              <span class="text-neutral-500">the AT Protocol.</span>
            </h1>
            <p class="text-xl text-neutral-400 font-light mb-12 max-w-2xl leading-relaxed">
              Master complex topics with spaced repetition, linked notes, and active recall. Share your decks, notes,
              and discoveries with the community.
            </p>
            <div class="flex gap-4">
              <A
                href="/login"
                class="bg-blue-600 hover:bg-blue-700 text-white px-8 py-4 font-medium text-lg transition-colors inline-flex items-center gap-2">
                Get Started
                <span class="text-xl">→</span>
              </A>
            </div>
          </div>
        </section>

        <section class="max-w-7xl mx-auto px-6 py-24">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <Feature
              title="Flashcards"
              desc="Built-in spaced repetition system (SRS) ensuring you review the right material at the right time." />
            <Feature
              title="Linked Notes"
              desc="Connect concepts with bidirectional links. Build a knowledge graph that grows with your understanding." />
            <Feature
              title="Lectures & Articles"
              desc="Import content directly. Highlight, annotate, and turn key insights into flashcards instantly." />
            <Feature
              title="Social Learning"
              desc="Publish your decks, follow curators, and fork existing content to improve it for everyone." />
            <Feature
              title="Local-First"
              desc="Your data lives on your device. Offline-first architecture with ATProto for decentralized sync." />
            <Feature
              title="Open Source"
              desc="Validates knowledge, not proprietary locks. Inspect the code, extend the schema, own the platform." />
          </div>
        </section>
      </main>

      <footer class="border-t border-[#393939] py-12 bg-[#161616]">
        <div class="max-w-7xl mx-auto px-6 text-[#C6C6C6] text-xs font-light flex flex-col md:flex-row justify-between items-center gap-4">
          <p>
            © 2025 Stormlight Labs. Made with ⚡️ by
            <a href="https://desertthunder.dev" target="_blank" class="hover:text-[#F4F4F4] transition-colors">
              Owais.
            </a>
          </p>
          <div class="flex gap-6">
            <a href="https://github.com/stormlightlabs" target="_blank" class="hover:text-[#F4F4F4] transition-colors">
              GitHub
            </a>
            <a href="#" class="hover:text-[#F4F4F4] transition-colors">Docs</a>
            <a href="#" class="hover:text-[#F4F4F4] transition-colors">Privacy</a>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Landing;
