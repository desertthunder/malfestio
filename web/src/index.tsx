/* @refresh reload */
import "@fontsource-variable/figtree";
import "@fontsource-variable/source-serif-4";
import { render } from "solid-js/web";
import "./index.css";
import App from "./App.tsx";

const root = document.getElementById("root");

render(() => <App />, root!);
