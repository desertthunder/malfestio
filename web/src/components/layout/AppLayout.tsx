import { Toaster } from "$ui/Toast";
import type { Component, JSX } from "solid-js";
import { Footer } from "./Footer";
import { Header } from "./Header";

export const AppLayout: Component<{ children?: JSX.Element }> = (props) => (
  <div class="min-h-screen bg-[#161616] text-[#F4F4F4] font-sans selection:bg-[#0F62FE]/30 flex flex-col">
    <Header />
    <main class="container mx-auto px-4 py-8 md:px-6 lg:px-8 max-w-7xl flex-1">{props.children}</main>
    <Footer />
    <Toaster />
  </div>
);
