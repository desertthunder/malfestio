import { authStore } from "./store";

const API_BASE = "http://localhost:8080/api";

export async function apiFetch(path: string, options: RequestInit = {}) {
  const token = authStore.accessJwt();

  const headers = new Headers(options.headers);
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  if (options.body && typeof options.body === "string" && !headers.has("Content-Type")) {
    headers.set("Content-Type", "application/json");
  }

  const response = await fetch(`${API_BASE}${path}`, { ...options, headers });

  if (response.status === 401) {
    authStore.logout();
    window.location.href = "/login";
  }

  return response;
}

export const api = {
  get: (path: string) => apiFetch(path, { method: "GET" }),
  post: (path: string, body: any) => apiFetch(path, { method: "POST", body: JSON.stringify(body) }),
};
