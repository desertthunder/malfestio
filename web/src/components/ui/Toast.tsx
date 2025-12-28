import { For, Match, Switch } from "solid-js";
import type { Component } from "solid-js";
import { toast, toasts, type ToastType } from "../../lib/toast";

const borderColors: Record<ToastType, string> = {
  success: "border-l-4 border-green-500",
  error: "border-l-4 border-red-500",
  warning: "border-l-4 border-yellow-500",
  info: "border-l-4 border-blue-500",
};

const InfoIcon: Component = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    class="h-6 w-6 text-blue-500"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor">
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
  </svg>
);

export const Toaster: Component = () => {
  return (
    <div class="fixed top-4 right-4 z-50 flex flex-col gap-2 w-full max-w-sm">
      <For each={toasts()}>
        {(t) => (
          <div
            class={`relative flex items-center gap-3 p-4 bg-gray-900 shadow-lg text-white transition-all transform animate-in slide-in-from-right-full duration-300 ${
              borderColors[t.type]
            }`}
            role="alert">
            <div class="flex-shrink-0">
              <Switch fallback={<InfoIcon />}>
                <Match when={t.type === "success"}>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-6 w-6 text-green-400"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                </Match>
                <Match when={t.type === "error"}>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-6 w-6 text-red-500"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </Match>
                <Match when={t.type === "warning"}>
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-6 w-6 text-yellow-500"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                </Match>
              </Switch>
            </div>
            <div class="flex-1 text-sm">{t.message}</div>
            <button
              onClick={() => toast.remove(t.id)}
              class="flex-shrink-0 text-gray-400 hover:text-white focus:outline-none"
              aria-label="Close">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-4 w-4"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        )}
      </For>
    </div>
  );
};
