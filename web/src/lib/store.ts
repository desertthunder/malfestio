import { createRoot, createSignal } from "solid-js";
import type { User } from "./model";

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
