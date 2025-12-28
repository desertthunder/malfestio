import { Route, Router } from "@solidjs/router";
import type { Component } from "solid-js";
import { Show } from "solid-js";
import { AppLayout } from "./components/layout/AppLayout";
import { authStore } from "./lib/store";
import DeckNew from "./pages/DeckNew";
import Home from "./pages/Home";
import Import from "./pages/Import";
import Landing from "./pages/Landing";
import Login from "./pages/Login";
import NoteNew from "./pages/NoteNew";

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

      {/* Protected Routes */}
      <Route path="/" component={() => <ProtectedRoute component={Home} />} />
      <Route path="/decks/new" component={() => <ProtectedRoute component={DeckNew} />} />
      <Route path="/notes/new" component={() => <ProtectedRoute component={NoteNew} />} />
      <Route path="/import" component={() => <ProtectedRoute component={Import} />} />

      {
        /* TODO: Catch-all or 404 */
      }
    </Router>
  );
};

export default App;
