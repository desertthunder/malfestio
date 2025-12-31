import { AppLayout } from "$components/layout/AppLayout";
import { api } from "$lib/api";
import { authStore } from "$lib/store";
import { useNavigate } from "@solidjs/router";
import type { Component } from "solid-js";
import { createSignal } from "solid-js";

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
      if (!password()) {
        const res = await api.startOAuth(identifier());
        if (!res.ok) {
          let errorMsg = "OAuth init failed";
          if ("json" in res) {
            const err = await res.json();
            errorMsg = err.error || errorMsg;
          }
          setError(errorMsg);
          setIsLoading(false);
        }

        return;
      }

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
    <AppLayout>
      <div class="min-h-[calc(100vh-8rem)] flex items-center justify-center p-4">
        <div class="w-full max-w-md bg-[#262626] border border-[#393939] p-8 shadow-lg section-entry">
          <h1 class="text-3xl font-light text-[#F4F4F4] mb-2 tracking-tight">Log in</h1>
          <p class="text-[#C6C6C6] text-sm mb-8 font-light">Continue to Malfestio</p>

          <form onSubmit={handleLogin} class="space-y-6">
            {error() && (
              <div class="bg-red-900/20 text-red-400 text-sm p-4 border-l-2 border-red-500 flex items-start gap-2 animate-in fade-in slide-in-from-top-2">
                <span class="font-bold">Error:</span> {error()}
              </div>
            )}

            <div class="space-y-2">
              <label class="block text-xs font-semibold text-[#8D8D8D] uppercase tracking-wider">Handle</label>
              <input
                type="text"
                value={identifier()}
                onInput={(e) => setIdentifier(e.currentTarget.value)}
                class="w-full bg-[#161616] border-b border-[#8D8D8D] focus:border-[#0F62FE] focus:outline-none p-4 transition-colors text-[#F4F4F4] placeholder-[#525252]"
                placeholder="user.bsky.social"
                required />
            </div>

            <div class="space-y-2">
              <div class="flex justify-between">
                <label class="block text-xs font-semibold text-[#8D8D8D] uppercase tracking-wider">
                  App Password (Optional)
                </label>
                <span class="text-xs text-[#8D8D8D] italic">Leave blank for OAuth</span>
              </div>
              <input
                type="password"
                value={password()}
                onInput={(e) => setPassword(e.currentTarget.value)}
                class="w-full bg-[#161616] border-b border-[#8D8D8D] focus:border-[#0F62FE] focus:outline-none p-4 transition-colors text-[#F4F4F4] placeholder-[#525252]"
                placeholder="••••••••" />
            </div>

            <div class="pt-4">
              <button
                type="submit"
                disabled={isLoading()}
                class="w-full bg-[#0F62FE] hover:bg-[#0353E9] text-white py-4 font-medium text-sm text-left px-4 flex justify-between items-center transition-colors disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-[#393939]">
                {isLoading() ? "Authenticating..." : (password() ? "Log in with Password" : "Continue with OAuth")}
                <span class="text-lg">→</span>
              </button>
            </div>
          </form>

          <div class="mt-8 text-xs text-[#8D8D8D] border-t border-[#393939] pt-4">
            <p>Use your Handle to log in via your PDS (OAuth), or provide an App Password directly.</p>
          </div>
        </div>
      </div>
    </AppLayout>
  );
};

export default Login;
