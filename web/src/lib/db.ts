import Dexie, { type EntityTable } from "dexie";
import type { CardType, Visibility } from "./model";

export type SyncStatus = "local_only" | "synced" | "pending_push" | "conflict";

type EntityKind = "deck" | "card" | "note";

type OperationKind = "push" | "delete";

type SyncTracking = { syncStatus: SyncStatus; localVersion: number; pdsCid?: string };

export type LocalDeck = SyncTracking & {
  id: string;
  ownerDid: string;
  title: string;
  description: string;
  tags: string[];
  visibility: Visibility;
  publishedAt?: string;
  forkOf?: string;
  pdsUri?: string;
  updatedAt: string;
};

export type LocalCard = SyncTracking & {
  id: string;
  deckId: string;
  front: string;
  back: string;
  mediaUrl?: string;
  cardType: CardType;
  hints: string[];
};

export type LocalNote = SyncTracking & {
  id: string;
  ownerDid: string;
  title: string;
  body: string;
  tags: string[];
  visibility: Visibility;
  publishedAt?: string;
  links: string[];
  pdsUri?: string;
  updatedAt: string;
};

export type SyncQueueItem = {
  id?: number;
  entityType: EntityKind;
  entityId: string;
  operation: OperationKind;
  createdAt: string;
  retryCount: number;
  lastError?: string;
};

class MalfestioDatabase extends Dexie {
  decks!: EntityTable<LocalDeck, "id">;
  cards!: EntityTable<LocalCard, "id">;
  notes!: EntityTable<LocalNote, "id">;
  syncQueue!: EntityTable<SyncQueueItem, "id">;

  constructor() {
    super("malfestio");

    this.version(1).stores({
      decks: "id, ownerDid, syncStatus, updatedAt",
      cards: "id, deckId, syncStatus",
      notes: "id, ownerDid, syncStatus, updatedAt",
      syncQueue: "++id, entityType, entityId, createdAt",
    });
  }
}

export const db = new MalfestioDatabase();

export function generateLocalId(): string {
  return `local_${Date.now()}_${Math.random().toString(36).slice(2, 11)}`;
}

export function isLocalId(id: string): boolean {
  return id.startsWith("local_");
}
