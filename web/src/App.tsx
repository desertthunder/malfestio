import { AppLayout } from "$components/layout/AppLayout";
import { authStore } from "$lib/store";
import DeckNew from "$pages/DeckNew";
import DeckView from "$pages/DeckView";
import Feed from "$pages/Feed";
import Home from "$pages/Home";
import Import from "$pages/Import";
import Landing from "$pages/Landing";
import LectureImport from "$pages/LectureImport";
import Login from "$pages/Login";
import NoteNew from "$pages/NoteNew";
import NotFound from "$pages/NotFound";
import Review from "$pages/Review";
import { Route, Router } from "@solidjs/router";
import type { Component } from "solid-js";
import { Show } from "solid-js";

const ProtectedRoute: Component<{ component: Component }> = (props) => {
  return (
    <Show when={authStore.isAuthenticated()} fallback={<Landing />}>
      <AppLayout>
        <props.component />
      </AppLayout>
    </Show>
  );
};

const App: Component = () => {
  return (
    <Router>
      <Route path="/login" component={Login} />
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
      <Route path="*" component={() => <ProtectedRoute component={NotFound} />} />
    </Router>
  );
};

export default App;
