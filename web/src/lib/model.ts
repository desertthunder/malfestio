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
