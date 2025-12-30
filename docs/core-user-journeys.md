# Core User Journeys

This document outlines the core user journeys and detailed user flows for Malfestio.

## 1. Import Source & Publish Deck

**Goal**: A creator builds a study deck from an external resource and shares it.

### High-Level Workflow

1. **Import**: User inputs a URL (Article) or pastes text.
2. **Generate**: System extracts metadata (and optionally snapshots content).
3. **Authoring**:
    - User highlights key sections in the source.
    - User creates **Notes** linked to highlights.
    - User generates **Cards** (Flashcards) from Notes or directly from source.
4. **Assembly**: User organizes Cards into a **Deck**.
5. **Publish**: User sets visibility (e.g., Public) and publishes the Deck.
6. **Result**: The Deck is now a shareable Artifact (ATProto record).

### Detailed Flows

#### Content Import

**Import Article**:

1. Header → "Import"
2. Enter article URL
3. Submit → article parsed, deck/note created

**Import Lecture**:

1. Import page → "Lecture Import" tab
2. Enter lecture URL
3. Submit → lecture content extracted

#### Note Management

**Create Note**:

1. Header → "Notes" → "New Note"
2. Fill: title, body (markdown), tags
3. Add wikilinks with `[[Note Title]]`
4. Set visibility
5. Submit → note created

**View Notes**:

1. Header → "Notes"
2. Browse notes with backlink navigation

#### Deck Management

**Create Deck**:

1. Library (`/`) → "Create Deck"
2. Fill: title, description, tags
3. Set visibility (Private/Unlisted/Public/SharedWith)
4. Add cards (front/back, optional hints, card type)
5. Submit → deck created, redirected to Library

**View Deck**:

1. Library → click deck card
2. View title, description, tags, card list
3. Options: Edit, Study, Back to Library

## 2. Daily Study Loop

**Goal**: A learner maintains their knowledge using Spaced Repetition (SRS).

### High-Level Workflow

1. **Session Start**: User opens the app/daily study mode.
2. **Review Queue**: System presents cards due for review based on SRS algorithm (e.g., SM-2).
3. **Interaction**:
    - User sees **Front** of card.
    - User attempts recall.
    - User reveals **Back**.
4. **Grading**: User self-grades (e.g., 0-5).
5. **Update**: System schedules next review interval.
6. **Progress**: User sees feedback (cards done, streak incremented).
    - *Note: All grading/progress data is strictly private.*

### Detailed Flows

#### Daily Review

1. Navigate to `/review` or click "Review" in header
2. View study stats: due count, streak, reviewed today
3. Click "Start Study Session"
4. Card front shown → press **Space** to flip
5. View answer → grade with **1-5** keys
6. Repeat until all due cards complete
7. View completion message and updated stats

#### Deck-Specific Review

1. Navigate to deck view (`/decks/:id`)
2. Click "Study Deck"
3. Review only cards from that deck
4. Same keyboard controls apply

#### Progress Tracking

- **Due count**: Cards needing review today

- **Streak**: Consecutive days studied
- **Reviewed today**: Cards completed this session
- **Interval growth**: SM-2 algorithm increases intervals for mastered cards

#### Keyboard Shortcuts

| Key   | Action         |
| ----- | -------------- |
| Space | Flip card      |
| 1     | Grade: Again   |
| 2     | Grade: Hard    |
| 3     | Grade: Good    |
| 4     | Grade: Easy    |
| 5     | Grade: Perfect |
| E     | Quick edit     |
| Esc   | Exit session   |

## 3. Social Collaboration (Follow/Fork)

**Goal**: A learner discovers content and improves it.

### High-Level Workflow

1. **Discovery**:
    - User follows a Curator.
    - User sees a new Deck in their "New from Follows" feed.
2. **Acquisition**: User saves/pins the Deck to their library.
3. **Contribution (Forking)**:
    - User identifies a gap or error in the Deck.
    - User **Forks** the Deck.
    - User edits cards or adds new ones.
    - User republishes the modified Deck (referencing the original).
4. **Loop**: Original author (or others) can see the fork and potentially merge changes (future scope) or users can switch to the better fork.

## 4. Discussion & Moderation

**Goal**: Community interaction while maintaining safety.

### High-Level Workflow

1. **Context**: A User is viewing a public Card or Deck.
2. **Discuss**: User adds a **Comment** (threaded) asking for clarification.
3. **Report** (Unhappy Path):
    - User encounters abusive content/spam.
    - User triggers **Report** flow.
    - Moderation system receives report.
    - Content may be hidden/labeled based on moderation actions.

## 5. Lecture Study Workflow

**Goal**: Deep study of long-form audio/video content.

### High-Level Workflow

1. **Import**: User provides a Lecture URL (e.g., YouTube/Video).
2. **Structure**:
    - User creates an **Outline** of the lecture.
    - User adds **Timestamps** to segment the content.
3. **Link**:
    - User creates Cards specific to timestamped segments.
    - Clicking context on a Card jumps video to the specific timestamp.

## Authentication

### Login

1. Navigate to `/login`
2. Enter Bluesky handle and app password
3. Submit → redirected to Library

### Logout

1. Click avatar in header → "Logout"
2. → redirected to Landing page

## 6. Onboarding & Personalization

**Goal**: New users get a personalized experience based on their learning goals.

### High-Level Workflow

1. **First Login**: User authenticates for the first time.
2. **Persona Selection**: User sees onboarding dialog with persona options:
   - **Learner**: Focus on studying existing content
   - **Creator**: Focus on building and sharing decks
   - **Curator**: Focus on discovering and organizing content
3. **Personalized Experience**: Empty states and tips adapt to chosen persona.
4. **Progress**: User preferences stored in backend for consistency across sessions.

### Detailed Flows

#### First-Time Onboarding

1. User logs in successfully
2. System fetches preferences from `/api/preferences`
3. If `onboarding_completed_at` is null, show OnboardingDialog
4. User selects persona → Submit
5. Backend stores persona and marks onboarding complete
6. Dialog closes, user sees personalized empty states

#### Persona-Aware Empty States

- **Home (Library)**: Tips and actions tailored to persona
    - Learners: "Browse Discovery" and "Fork decks you like"
    - Creators: "Create New Deck" and "Import from Article"
    - Curators: "View Feed" and "Follow creators"

- **Review**: First-timer guidance explaining SRS for users with no reviews

## 7. Help & Support

**Goal**: Users can find answers to common questions.

### Detailed Flows

#### Accessing Help

1. Footer → "Help" link, or navigate to `/help`
2. View FAQ organized by category:
   - Getting Started
   - Spaced Repetition
   - AT Protocol & Privacy
   - Community & Sharing
3. Click questions to expand accordion answers

#### Beta Notice

- Help page displays prominent notice that Malfestio is in active development
- Links to Bluesky and GitHub for community support
