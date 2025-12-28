import { Route, Router } from "@solidjs/router";
import type { Component } from "solid-js";
import { Show } from "solid-js";
import { AppLayout } from "./components/layout/AppLayout";
import { authStore } from "./lib/store";
import DeckNew from "./pages/DeckNew";
import Home from "./pages/Home";
import Landing from "./pages/Landing";
import Login from "./pages/Login";

const Root: Component = () => {
  return (
    <Show when={authStore.isAuthenticated()} fallback={<Landing />}>
      <AppLayout>
        <Router>
          <Route path="/" component={Home} />
          <Route path="/decks/new" component={DeckNew} />
        </Router>
      </AppLayout>
    </Show>
  );
};

const App: Component = () => {
  return (
    <Router>
      <Route path="/" component={Root} />
      <Route path="/login" component={Login} />
    </Router>
  );
};

export default App;
