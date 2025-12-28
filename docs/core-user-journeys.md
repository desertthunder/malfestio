# Core User Journeys

This document outlines the five core user journeys for the initial product version.

## 1. Import Source & Publish Deck

**Goal**: A creator builds a study deck from an external resource and shares it.

1. **Import**: User inputs a URL (Article) or pastes text.
2. **Generate**: System extracts metadata (and optionally snapshots content).
3. **Authoring**:
    * User highlights key sections in the source.
    * User creates **Notes** linked to highlights.
    * User generates **Cards** (Flashcards) from Notes or directly from source.
4. **Assembly**: User organizes Cards into a **Deck**.
5. **Publish**: User sets visibility (e.g., Public) and publishes the Deck.
6. **Result**: The Deck is now a shareable Artifact (ATProto record).

## 2. Daily Study Loop

**Goal**: A learner maintains their knowledge using Spaced Repetition (SRS).

1. **Session Start**: User opens the app/daily study mode.
2. **Review Queue**: System presents cards due for review based on SRS algorithm (e.g., SM-2).
3. **Interaction**:
    * User sees **Front** of card.
    * User attempts recall.
    * User reveals **Back**.
4. **Grading**: User self-grades (e.g., 0-5).
5. **Update**: System schedules next review interval.
6. **Progress**: User sees feedback (cards done, streak incremented).
    * *Note: All grading/progress data is strictly private.*

## 3. Social Collaboration (Follow/Fork)

**Goal**: A learner discovers content and improves it.

1. **Discovery**:
    * User follows a Curator.
    * User sees a new Deck in their "New from Follows" feed.
2. **Acquisition**: User saves/pins the Deck to their library.
3. **Contribution (Forking)**:
    * User identifies a gap or error in the Deck.
    * User **Forks** the Deck.
    * User edits cards or adds new ones.
    * User republishes the modified Deck (referencing the original).
4. **Loop**: Original author (or others) can see the fork and potentially merge changes (future scope) or users can switch to the better fork.

## 4. Discussion & Moderation

**Goal**: Community interaction while maintaining safety.

1. **Context**: A User is viewing a public Card or Deck.
2. **Discuss**: User adds a **Comment** (threaded) asking for clarification.
3. **Report** (Unhappy Path):
    * User encounters abusive content/spam.
    * User triggers **Report** flow.
    * Moderation system receives report.
    * Content may be hidden/labeled based on moderation actions.

## 5. Lecture Study Workflow

**Goal**: Deep study of long-form audio/video content.

1. **Import**: User provides a Lecture URL (e.g., YouTube/Video).
2. **Structure**:
    * User creates an **Outline** of the lecture.
    * User adds **Timestamps** to segment the content.
3. **Link**:
    * User creates Cards specific to timestamped segments.
    * Clicking context on a Card jumps video to the specific timestamp.
