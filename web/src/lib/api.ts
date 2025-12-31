import type { CreateDeckPayload } from "./model";
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
  getStats: () => apiFetch("/review/stats", { method: "GET" }),
  follow: (did: string) => apiFetch(`/social/follow/${did}`, { method: "POST" }),
  unfollow: (did: string) => apiFetch(`/social/unfollow/${did}`, { method: "POST" }),
  getFollowers: (did: string) => apiFetch(`/social/followers/${did}`, { method: "GET" }),
  getFollowing: (did: string) => apiFetch(`/social/following/${did}`, { method: "GET" }),
  getComments: (deckId: string) => apiFetch(`/decks/${deckId}/comments`, { method: "GET" }),
  getFeedFollows: () => apiFetch("/feeds/follows", { method: "GET" }),
  getFeedTrending: () => apiFetch("/feeds/trending", { method: "GET" }),
  forkDeck: (deckId: string) => apiFetch(`/decks/${deckId}/fork`, { method: "POST" }),
  getDecks: () => apiFetch("/decks", { method: "GET" }),
  getDeck: (id: string) => apiFetch(`/decks/${id}`, { method: "GET" }),
  getDeckCards: (id: string) => apiFetch(`/decks/${id}/cards`, { method: "GET" }),
  getPreferences: () => apiFetch("/preferences", { method: "GET" }),
  getDiscovery: () => apiFetch("/discovery", { method: "GET" }),
  getUserProfile: (did: string) => apiFetch(`/users/${did}/profile`, { method: "GET" }),
  getRemoteDeck: (uri: string) => apiFetch(`/remote/deck?uri=${encodeURIComponent(uri)}`, { method: "GET" }),
  exportData: (collection: "decks" | "notes") => apiFetch(`/export/${collection}`, { method: "GET" }),
  createDeck: async (payload: CreateDeckPayload) => {
    const { cards, ...deckPayload } = payload;
    const res = await apiFetch("/decks", { method: "POST", body: JSON.stringify(deckPayload) });
    if (!res.ok) return res;

    const deck = await res.json();
    if (cards && cards.length > 0) {
      await Promise.all(cards.map((c) =>
        apiFetch("/cards", {
          method: "POST",
          body: JSON.stringify({ deck_id: deck.id, front: c.front, back: c.back, media_url: c.mediaUrl }),
        })
      ));
    }

    return {
      ok: true,
      json: async () => deck,
    };
  },
  addComment: (deckId: string, content: string, parentId?: string) => {
    return apiFetch(`/decks/${deckId}/comments`, {
      method: "POST",
      body: JSON.stringify({ content, parent_id: parentId }),
    });
  },
  search: (query: string, limit = 20, offset = 0, source?: "local" | "remote") => {
    const params = new URLSearchParams({ q: query, limit: String(limit), offset: String(offset) });
    if (source) params.set("source", source);
    return apiFetch(`/search?${params}`, { method: "GET" });
  },
  getDueCards: (deckId?: string, limit = 20) => {
    const params = new URLSearchParams({ limit: String(limit) });
    if (deckId) params.set("deck_id", deckId);
    return apiFetch(`/review/due?${params}`, { method: "GET" });
  },
  submitReview: (cardId: string, grade: number) => {
    return apiFetch("/review/submit", { method: "POST", body: JSON.stringify({ card_id: cardId, grade }) });
  },
  updatePreferences: (updates: import("./model").UpdatePreferencesPayload) => {
    return apiFetch("/preferences", { method: "PUT", body: JSON.stringify(updates) });
  },
  startOAuth: async (handle: string) => {
    const res = await apiFetch("/oauth/authorize", { method: "POST", body: JSON.stringify({ handle }) });
    if (res.ok) {
      const data = await res.json();
      window.location.href = data.authorization_url;
      return { ok: true };
    }
    return res;
  },
};
