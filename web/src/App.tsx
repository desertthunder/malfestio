import { AppLayout } from "$components/layout/AppLayout";
import { OnboardingDialog } from "$components/OnboardingDialog";
import type { Persona } from "$lib/model";
import { authStore, prefStore } from "$lib/store";
import About from "$pages/About";
import DeckNew from "$pages/DeckNew";
import DeckView from "$pages/DeckView";
import Discovery from "$pages/Discovery";
import Feed from "$pages/Feed";
import Help from "$pages/Help";
import Home from "$pages/Home";
import Import from "$pages/Import";
import Landing from "$pages/Landing";
import LectureImport from "$pages/LectureImport";
import Login from "$pages/Login";
import NoteNew from "$pages/NoteNew";
import NotFound from "$pages/NotFound";
import Review from "$pages/Review";
import Search from "$pages/Search";
import { Route, Router } from "@solidjs/router";
import type { Component } from "solid-js";
import { createEffect, createSignal, onMount, Show } from "solid-js";

const ProtectedRoute: Component<{ component: Component }> = (props) => {
  const [showOnboarding, setShowOnboarding] = createSignal(false);

  onMount(async () => {
    if (authStore.isAuthenticated()) {
      await prefStore.fetchPrefs();
    }
  });

  createEffect(() => {
    if (prefStore.needsOnboarding()) {
      setShowOnboarding(true);
    }
  });

  const handleOnboardingComplete = (_persona: Persona) => {
    setShowOnboarding(false);
    prefStore.fetchPrefs();
  };

  return (
    <Show when={authStore.isAuthenticated()} fallback={<Landing />}>
      <AppLayout>
        <props.component />
      </AppLayout>
      <OnboardingDialog open={showOnboarding()} onComplete={handleOnboardingComplete} />
    </Show>
  );
};

const App: Component = () => {
  return (
    <Router>
      <Route path="/login" component={Login} />
      <Route path="/about" component={About} />
      <Route path="/help" component={Help} />
      <Route path="/" component={() => <ProtectedRoute component={Home} />} />
      <Route path="/decks" component={() => <ProtectedRoute component={Home} />} />
      <Route path="/decks/new" component={() => <ProtectedRoute component={DeckNew} />} />
      <Route path="/notes/new" component={() => <ProtectedRoute component={NoteNew} />} />
      <Route path="/decks/:id" component={() => <ProtectedRoute component={DeckView} />} />
      <Route path="/import" component={() => <ProtectedRoute component={Import} />} />
      <Route path="/import/lecture" component={() => <ProtectedRoute component={LectureImport} />} />
      <Route path="/review" component={() => <ProtectedRoute component={Review} />} />
      <Route path="/review/:deckId" component={() => <ProtectedRoute component={Review} />} />
      <Route path="/feed" component={() => <ProtectedRoute component={Feed} />} />
      <Route path="/search" component={() => <ProtectedRoute component={Search} />} />
      <Route path="/discovery" component={() => <ProtectedRoute component={Discovery} />} />
      <Route path="*" component={() => <ProtectedRoute component={NotFound} />} />
    </Router>
  );
};

export default App;
