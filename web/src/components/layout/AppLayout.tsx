import { Toaster } from "$ui/Toast";
import type { Component, JSX } from "solid-js";
import { Header } from "./Header";

type AppLayoutProps = { children?: JSX.Element };

export const AppLayout: Component<AppLayoutProps> = (props) => {
  return (
    <div class="min-h-screen bg-[#161616] text-[#F4F4F4] font-sans selection:bg-[#0F62FE]/30">
      <Header />
      <main class="container mx-auto px-4 py-8 md:px-6 lg:px-8 max-w-7xl">{props.children}</main>
      <Toaster />
    </div>
  );
};
