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
  const [accessJwt, setAccessJwt] = createSignal<string | null>(localStorage.getItem("accessJwt"));
  const [_refreshJwt, setRefreshJwt] = createSignal<string | null>(localStorage.getItem("refreshJwt"));

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

  return { user, accessJwt, isAuthenticated: () => !!accessJwt(), login, logout };
}

export const authStore = createRoot(createAuthStore);

function createPreferencesStore() {
  const [preferences, setPreferences] = createSignal<UserPreferences | null>(null);
  const [loading, setLoading] = createSignal(false);

  const fetchPreferences = async () => {
    if (!authStore.isAuthenticated()) return;
    setLoading(true);
    try {
      const res = await api.getPreferences();
      if (res.ok) {
        setPreferences(await res.json());
      }
    } catch (e) {
      console.error("Failed to fetch preferences:", e);
    } finally {
      setLoading(false);
    }
  };

  const updatePreferences = async (updates: { persona?: Persona; complete_onboarding?: boolean }) => {
    try {
      const res = await api.updatePreferences(updates);
      if (res.ok) {
        setPreferences(await res.json());
      }
    } catch (e) {
      console.error("Failed to update preferences:", e);
    }
  };

  const needsOnboarding = () => {
    const prefs = preferences();
    return prefs !== null && prefs.onboarding_completed_at === null;
  };

  const persona = () => preferences()?.persona ?? null;

  return { preferences, loading, fetchPreferences, updatePreferences, needsOnboarding, persona };
}

export const preferencesStore = createRoot(createPreferencesStore);
