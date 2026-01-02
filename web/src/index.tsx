/* @refresh reload */
import "@fontsource-variable/figtree";
import "@fontsource-variable/jetbrains-mono";
import "@fontsource/google-sans-code";
import "@fontsource/monaspace-argon";
import "@fontsource/monaspace-krypton";
import "@fontsource/monaspace-neon";
import "@fontsource/monaspace-radon";
import "@fontsource/monaspace-xenon";
import { render } from "solid-js/web";
import "./index.css";
import App from "./App.tsx";

const root = document.getElementById("root");

render(() => <App />, root!);
