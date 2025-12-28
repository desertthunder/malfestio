import { useNavigate } from "@solidjs/router";
import type { Component } from "solid-js";
import { createSignal } from "solid-js";
import { api } from "../lib/api";
import { authStore } from "../lib/store";

const Login: Component = () => {
  const [identifier, setIdentifier] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [error, setError] = createSignal("");
  const [isLoading, setIsLoading] = createSignal(false);
  const navigate = useNavigate();

  const handleLogin = async (e: Event) => {
    e.preventDefault();
    setIsLoading(true);
    setError("");

    try {
      const response = await api.post("/auth/login", { identifier: identifier(), password: password() });

      if (response.ok) {
        const data = await response.json();
        authStore.login(data);
        navigate("/");
      } else {
        const err = await response.json();
        setError(err.error || "Login failed");
      }
    } catch {
      setError("Network error or server unreachable");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div class="min-h-[calc(100vh-4rem)] flex items-center justify-center bg-neutral-100 dark:bg-black p-4">
      <div class="w-full max-w-md bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800 p-8 shadow-sm">
        <h1 class="text-3xl font-light text-neutral-900 dark:text-white mb-8 tracking-tight">Login</h1>

        <form onSubmit={handleLogin} class="space-y-6">
          {error() && (
            <div class="bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 text-sm p-4 border border-red-200 dark:border-red-900/50">
              {error()}
            </div>
          )}

          <div class="space-y-2">
            <label class="block text-xs font-semibold text-neutral-500 uppercase tracking-wider">Handle</label>
            <input
              type="text"
              value={identifier()}
              onInput={(e) => setIdentifier(e.currentTarget.value)}
              class="w-full bg-neutral-100 dark:bg-neutral-800 border-b border-neutral-400 dark:border-neutral-600 focus:border-blue-500 focus:outline-none p-3 transition-colors text-neutral-900 dark:text-white rounded-t-sm"
              placeholder="user.bsky.social"
              required />
          </div>

          <div class="space-y-2">
            <label class="block text-xs font-semibold text-neutral-500 uppercase tracking-wider">App Password</label>
            <input
              type="password"
              value={password()}
              onInput={(e) => setPassword(e.currentTarget.value)}
              class="w-full bg-neutral-100 dark:bg-neutral-800 border-b border-neutral-400 dark:border-neutral-600 focus:border-blue-500 focus:outline-none p-3 transition-colors text-neutral-900 dark:text-white rounded-t-sm"
              placeholder="••••••••"
              required />
          </div>

          <div class="pt-4">
            <button
              type="submit"
              disabled={isLoading()}
              class="w-full bg-neutral-900 dark:bg-white text-white dark:text-neutral-900 hover:bg-neutral-800 dark:hover:bg-neutral-100 py-4 font-medium text-sm text-left px-6 flex justify-between items-center transition-colors disabled:opacity-50 disabled:cursor-not-allowed">
              {isLoading() ? "Authenticating..." : "Continue"}
              <span class="text-lg">→</span>
            </button>
          </div>
        </form>

        <div class="mt-8 text-xs text-neutral-500 dark:text-neutral-400">
          <p class="mt-2">Use your BlueSky App Password, not your main password.</p>
        </div>
      </div>
    </div>
  );
};

export default Login;
