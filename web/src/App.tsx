import { AppLayout } from "$components/layout/AppLayout";
import { OnboardingDialog } from "$components/OnboardingDialog";
import type { Persona } from "$lib/model";
import { authStore, prefStore } from "$lib/store";
import About from "$pages/About";
import DeckNew from "$pages/DeckNew";
import DeckPreview from "$pages/DeckPreview";
import DeckView from "$pages/DeckView";
import Discovery from "$pages/Discovery";
import Feed from "$pages/Feed";
import Help from "$pages/Help";
import Home from "$pages/Home";
import Import from "$pages/Import";
import Landing from "$pages/Landing";
import LectureImport from "$pages/LectureImport";
import Library from "$pages/Library";
import Login from "$pages/Login";
import NoteNew from "$pages/NoteNew";
import NotFound from "$pages/NotFound";
import Review from "$pages/Review";
import Search from "$pages/Search";
import Settings from "$pages/Settings";
import { Route, Router } from "@solidjs/router";
import type { Component, ParentComponent } from "solid-js";
import { createEffect, createSignal, onMount, Show } from "solid-js";

const ProtectedLayout: ParentComponent = (props) => {
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
      <AppLayout>{props.children}</AppLayout>
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
      <Route path="/" component={ProtectedLayout}>
        <Route path="/" component={Home} />
        <Route path="/decks" component={Home} />
        <Route path="/decks/new" component={DeckNew} />
        <Route path="/notes/new" component={NoteNew} />
        <Route path="/decks/:id" component={DeckView} />
        <Route path="/import" component={Import} />
        <Route path="/import/lecture" component={LectureImport} />
        <Route path="/review" component={Review} />
        <Route path="/review/:deckId" component={Review} />
        <Route path="/feed" component={Feed} />
        <Route path="/search" component={Search} />
        <Route path="/discovery" component={Discovery} />
        <Route path="/library" component={Library} />
        <Route path="/library/preview" component={DeckPreview} />
        <Route path="/settings" component={Settings} />
        <Route path="*" component={NotFound} />
      </Route>
    </Router>
  );
};

export default App;
