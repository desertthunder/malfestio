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
import LoginSuccess from "$pages/LoginSuccess";
import NoteNew from "$pages/NoteNew";
import Notes from "$pages/Notes";
import NoteView from "$pages/NoteView";
import NotFound from "$pages/NotFound";
import Review from "$pages/Review";
import Search from "$pages/Search";
import Settings from "$pages/Settings";
import { Route, Router } from "@solidjs/router";
import type { Component, ParentComponent } from "solid-js";
import { createEffect, createSignal, onMount, Show } from "solid-js";
import { Motion, Presence } from "solid-motionone";

const LoadingScreen: Component = () => (
  <Motion.div
    exit={{ opacity: 0 }}
    transition={{ duration: 0.5 }}
    class="fixed inset-0 bg-[#161616] flex items-center justify-center z-50">
    <div class="text-[#0F62FE] animate-spin text-4xl w-12 h-12">
      <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24">
        <g fill="none" fill-rule="evenodd">
          <path d="m12.593 23.258l-.011.002l-.071.035l-.02.004l-.014-.004l-.071-.035q-.016-.005-.024.005l-.004.01l-.017.428l.005.02l.01.013l.104.074l.015.004l.012-.004l.104-.074l.012-.016l.004-.017l-.017-.427q-.004-.016-.017-.018m.265-.113l-.013.002l-.185.093l-.01.01l-.003.011l.018.43l.005.012l.008.007l.201.093q.019.005.029-.008l.004-.014l-.034-.614q-.005-.018-.02-.022m-.715.002a.02.02 0 0 0-.027.006l-.006.014l-.034.614q.001.018.017.024l.015-.002l.201-.093l.01-.008l.004-.011l.017-.43l-.003-.012l-.01-.01z" />
          <path
            fill="currentColor"
            d="M12 4.5a7.5 7.5 0 1 0 0 15a7.5 7.5 0 0 0 0-15M1.5 12C1.5 6.201 6.201 1.5 12 1.5S22.5 6.201 22.5 12S17.799 22.5 12 22.5S1.5 17.799 1.5 12"
            opacity="0.1" />
          <path
            fill="currentColor"
            d="M12 4.5a7.46 7.46 0 0 0-5.187 2.083a1.5 1.5 0 0 1-2.075-2.166A10.46 10.46 0 0 1 12 1.5a1.5 1.5 0 0 1 0 3" />
        </g>
      </svg>
    </div>
  </Motion.div>
);

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
    <Presence>
      <Show when={authStore.loading()}>
        <LoadingScreen />
      </Show>
      <Show when={!authStore.loading()}>
        <Show when={authStore.isAuthenticated()} fallback={<Landing />}>
          <AppLayout>{props.children}</AppLayout>
          <OnboardingDialog open={showOnboarding()} onComplete={handleOnboardingComplete} />
        </Show>
      </Show>
    </Presence>
  );
};

const App: Component = () => {
  return (
    <Router>
      <Route path="/login" component={Login} />
      <Route path="/login/success" component={LoginSuccess} />
      <Route path="/about" component={About} />
      <Route path="/help" component={Help} />
      <Route path="/" component={ProtectedLayout}>
        <Route path="/" component={Home} />
        <Route path="/decks" component={Home} />
        <Route path="/decks/new" component={DeckNew} />
        <Route path="/notes/new" component={NoteNew} />
        <Route path="/notes/:id" component={NoteView} />
        <Route path="/notes" component={Notes} />
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
