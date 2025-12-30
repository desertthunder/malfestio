import { authStore } from "./store";

const API_BASE = "/api";

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
  post: (path: string, body: unknown) => apiFetch(path, { method: "POST", body: JSON.stringify(body) }),
  getDueCards: (deckId?: string, limit = 20) => {
    const params = new URLSearchParams({ limit: String(limit) });
    if (deckId) params.set("deck_id", deckId);
    return apiFetch(`/review/due?${params}`, { method: "GET" });
  },
  submitReview: (cardId: string, grade: number) =>
    apiFetch("/review/submit", { method: "POST", body: JSON.stringify({ card_id: cardId, grade }) }),
  getStats: () => apiFetch("/review/stats", { method: "GET" }),
  follow: (did: string) => apiFetch(`/social/follow/${did}`, { method: "POST" }),
  unfollow: (did: string) => apiFetch(`/social/unfollow/${did}`, { method: "POST" }),
  getFollowers: (did: string) => apiFetch(`/social/followers/${did}`, { method: "GET" }),
  getFollowing: (did: string) => apiFetch(`/social/following/${did}`, { method: "GET" }),
  addComment: (deckId: string, content: string, parentId?: string) =>
    apiFetch(`/decks/${deckId}/comments`, { method: "POST", body: JSON.stringify({ content, parent_id: parentId }) }),
  getComments: (deckId: string) => apiFetch(`/decks/${deckId}/comments`, { method: "GET" }),
  getFeedFollows: () => apiFetch("/feeds/follows", { method: "GET" }),
  getFeedTrending: () => apiFetch("/feeds/trending", { method: "GET" }),
  forkDeck: (deckId: string) => apiFetch(`/decks/${deckId}/fork`, { method: "POST" }),
};
