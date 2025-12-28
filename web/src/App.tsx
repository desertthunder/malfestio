import { Route, Router } from "@solidjs/router";
import type { Component } from "solid-js";
import { AppLayout } from "./components/layout/AppLayout";
import Home from "./pages/Home";
import Login from "./pages/Login";

const App: Component = () => {
  return (
    <Router root={AppLayout}>
      <Route path="/" component={Home} />
      <Route path="/login" component={Login} />
    </Router>
  );
};

export default App;
