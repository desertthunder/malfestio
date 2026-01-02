export type User = { did: string; handle: string };

export type Visibility = { type: "Private" } | { type: "Unlisted" } | { type: "Public" } | {
  type: "SharedWith";
  content: string[];
};

export type CardType = "basic" | "cloze";

export type Card = {
  id?: string;
  front: string;
  back: string;
  mediaUrl?: string;
  cardType?: CardType;
  hints?: string[];
};

export type Deck = {
  id: string;
  owner_did: string;
  title: string;
  description: string;
  tags: string[];
  visibility: Visibility;
  published_at?: string;
  fork_of?: string;
};

export type Note = {
  id: string;
  owner_did: string;
  title: string;
  body: string;
  tags: string[];
  visibility: Visibility;
  published_at?: string;
  created_at: string;
  updated_at: string;
};

export type CreateDeckPayload = {
  title: string;
  description: string;
  tags: string[];
  visibility: Visibility;
  cards: Card[];
};

export type Grade = 0 | 1 | 2 | 3 | 4 | 5;

export type ReviewCard = {
  review_id: string;
  card_id: string;
  deck_id: string;
  deck_title: string;
  front: string;
  back: string;
  media_url?: string;
  hints: string[];
  due_at: string;
};

export type StudyStats = {
  due_count: number;
  current_streak: number;
  longest_streak: number;
  reviewed_today: number;
  total_reviews: number;
};

export type ReviewResponse = { ease_factor: number; interval_days: number; repetitions: number; due_at: string };

export type Comment = {
  id: string;
  deck_id: string;
  author_did: string;
  content: string;
  parent_id: string | null;
  created_at: string;
};

export type CommentNode = { comment: Comment; children: CommentNode[] };

export type FeedFollows = { decks: Deck[] };

export type SearchResult = {
  item_type: "deck";
  item_id: string;
  creator_did: string;
  data: Deck;
  rank: number;
  source?: "local" | "remote";
} | {
  item_type: "card";
  item_id: string;
  creator_did: string;
  data: Card & { deck_id: string };
  rank: number;
  source?: "local" | "remote";
} | {
  item_type: "note";
  item_id: string;
  creator_did: string;
  data: { id: string; title: string; owner_did: string };
  rank: number;
  source?: "local" | "remote";
};

export const asDeck = (r: SearchResult) => (r.item_type === "deck" ? r : undefined);
export const asCard = (r: SearchResult) => (r.item_type === "card" ? r : undefined);
export const asNote = (r: SearchResult) => (r.item_type === "note" ? r : undefined);

export type Persona = "learner" | "creator" | "curator";

export type UserPreferences = {
  user_did: string;
  persona: Persona | null;
  onboarding_completed_at: string | null;
  tutorial_deck_completed: boolean;
  density_mode: "compact" | "comfortable" | "spacious" | null;
};

export type UserProfile = {
  did: string;
  follower_count: number;
  following_count: number;
  deck_count: number;
  indexed_deck_count: number;
};

export type UpdatePreferencesPayload = {
  persona?: Persona;
  complete_onboarding?: boolean;
  tutorial_deck_completed?: boolean;
};
