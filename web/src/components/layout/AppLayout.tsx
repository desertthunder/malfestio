import type { Component, JSX } from "solid-js";
import { Header } from "./Header";

interface AppLayoutProps {
  children?: JSX.Element;
}

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <div class="min-h-screen bg-black text-gray-100 font-sans selection:bg-blue-500/30">
      <Header />
      <main class="container mx-auto px-4 py-8 md:px-6 lg:px-8 max-w-7xl">{props.children}</main>
    </div>
  );
};
