import { createRoot, createSignal } from "solid-js";
import { api } from "./api";
import type { Persona, User, UserPreferences } from "./model";

export type AuthState = {
  user: User | null;
  accessJwt: string | null;
  refreshJwt: string | null;
  isAuthenticated: boolean;
};

function createAuthStore() {
  const [user, setUser] = createSignal<User | null>(
    localStorage.getItem("did")
      ? { did: localStorage.getItem("did")!, handle: localStorage.getItem("handle") || "" }
      : null,
  );
  const [accessJwt, setAccessJwt] = createSignal(localStorage.getItem("accessJwt"));
  const [_, setRefreshJwt] = createSignal(localStorage.getItem("refreshJwt"));

  const login = (data: { accessJwt: string; refreshJwt: string; did: string; handle: string }) => {
    setAccessJwt(data.accessJwt);
    setRefreshJwt(data.refreshJwt);
    setUser({ did: data.did, handle: data.handle });

    localStorage.setItem("accessJwt", data.accessJwt);
    localStorage.setItem("refreshJwt", data.refreshJwt);
    localStorage.setItem("did", data.did);
    localStorage.setItem("handle", data.handle);
  };

  const logout = () => {
    setUser(null);
    setAccessJwt(null);
    setRefreshJwt(null);
    localStorage.clear();
  };

  const [loading, setLoading] = createSignal(true);

  const init = async () => {
    await new Promise((resolve) => setTimeout(resolve, 1500));
    setLoading(false);
  };

  init();

  return { user, accessJwt, isAuthenticated: () => !!accessJwt(), login, logout, loading };
}

export const authStore = createRoot(createAuthStore);

function createPrefStore() {
  const [prefs, setPrefs] = createSignal<UserPreferences | null>(null);
  const [loading, setLoading] = createSignal(false);

  const fetchPrefs = async () => {
    if (!authStore.isAuthenticated()) return;
    setLoading(true);
    try {
      const res = await api.getPreferences();
      if (res.ok) {
        setPrefs(await res.json());
      }
    } catch (e) {
      console.error("Failed to fetch preferences:", e);
    } finally {
      setLoading(false);
    }
  };

  const updatePreferences = async (
    updates: {
      persona?: Persona;
      complete_onboarding?: boolean;
      density_mode?: "compact" | "comfortable" | "spacious";
    },
  ) => {
    try {
      const res = await api.updatePreferences(updates);
      if (res.ok) {
        setPrefs(await res.json());
      }
    } catch (e) {
      console.error("Failed to update preferences:", e);
    }
  };

  const needsOnboarding = () => {
    const preferences = prefs();
    return preferences !== null && preferences.onboarding_completed_at === null;
  };

  const persona = () => prefs()?.persona ?? null;
  const densityMode = () => prefs()?.density_mode ?? "comfortable";
  return { prefs, loading, fetchPrefs, updatePreferences, needsOnboarding, persona, densityMode };
}

export const prefStore = createRoot(createPrefStore);
