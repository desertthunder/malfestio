import { Route, Router } from "@solidjs/router";
import type { Component } from "solid-js";
import { AppLayout } from "./components/layout/AppLayout";
import Home from "./pages/Home";

const App: Component = () => {
  return (
    <Router root={AppLayout}>
      <Route path="/" component={Home} />
    </Router>
  );
};

export default App;
