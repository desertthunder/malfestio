import type { Component } from "solid-js";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";

const Home: Component = () => {
  return (
    <div class="space-y-8">
      <section class="text-center py-16 space-y-6">
        <h1 class="text-5xl font-extrabold tracking-tight text-white sm:text-6xl bg-clip-text text-transparent bg-gradient-to-r from-blue-400 to-emerald-400">
          Learn Together
        </h1>
        <p class="text-xl text-gray-400 max-w-2xl mx-auto">
          Malfestio is a social learning platform built on the AT Protocol.
        </p>
        <div class="flex items-center justify-center gap-4 pt-4">
          <Button size="lg" variant="primary">Get Started</Button>
          <Button size="lg" variant="ghost">Learn More</Button>
        </div>
      </section>

      <section class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card title="Decks">
          <p class="mb-4">Create and manage your flashcard decks. Import from articles and lectures.</p>
          <Button variant="secondary" size="sm">View Decks</Button>
        </Card>
        <Card title="Review">
          <p class="mb-4">Daily review sessions optimized by the SM-2 algorithm to maximize retention.</p>
          <Button variant="secondary" size="sm">Start Review</Button>
        </Card>
        <Card title="Community">
          <p class="mb-4">Discover shared decks and follow other learners in the network.</p>
          <Button variant="secondary" size="sm">Explore</Button>
        </Card>
      </section>
    </div>
  );
};

export default Home;
