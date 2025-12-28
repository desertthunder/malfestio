# Information Architecture

This document defines the application structure, navigation, and data model mapping for Milestone A.

## Site Map (Navigation)

### Global Navigation

- **Home/Feed**: Discover content (Following, Trending).
- **Library**: User's saved Decks, curated Collections, and private progress.
- **Create**: Entry point for Authoring tools.
- **Search**: Global search for Decks, Curators, and Tags.
- **Profile**: User identity and settings.

### Logical View Hierarchy

- **/** (Home)
    - Feed of followed curators
- **/library**
    - Study Queue (What to review today)
    - My Decks (Created & Forked)
    - Saved Collections
- **/deck/:did/:slug** (Deck View)
    - Overview (Description, Stats)
    - Browser/Cards List
    - Study Mode (Launch Session)
- **/note/:rkey** (Note View)
    - Note Content (Markdown)
    - Backlinks / Linked Cards
- **/source/:rkey** (Article/Lecture View)
    - Source Metadata (Title, URL)
    - Snapshot/Content View (if saved)
    - Highlights List
- **/study/:session_id** (Study Session)
    - Active Recall Interface (Front/Back)
- **/editor**
    - New Deck / Edit Deck
    - Import Source (URL/Text)
- **/profile/:handle**
    - Public Decks
    - Followers/Following

## Data Model Mapping

Mapping screens to underlying data entities (Lexicon Records + Private State).

| Screen / Component | Primary Data Entity            | Secondary Entities                        | Private/Public                |
| :----------------- | :----------------------------- | :---------------------------------------- | :---------------------------- |
| **Deck Overview**  | `app.malfestio.deck`           | `app.malfestio.card` (refs), User Profile | **Public**                    |
| **Study Session**  | N/A (Ephemeral)                | `app.malfestio.card`, Private Review Log  | **Private**                   |
| **Card View**      | `app.malfestio.card`           | `app.malfestio.note`, Media Blobs         | **Public**                    |
| **Editor**         | Draft State (Local)            | Source (`article`), `note`                | **Private (Draft) -> Public** |
| **Source View**    | `app.malfestio.source.article` | `app.malfestio.note` (linked)             | **Public**                    |
| **Note View**      | `app.malfestio.note`           | Backlinks (`card`/`deck`)                 | **Public**                    |
| **Library**        | `app.malfestio.collection`     | Bookmarks, User Prefs                     | **Mixed**                     |
| **Comments**       | `app.malfestio.thread.comment` | User Profile                              | **Public**                    |

## URL Structure

- `https://app.example.com/` - Home
- `https://app.example.com/profile/<handle>` - User Profile
- `https://app.example.com/profile/<handle>/deck/<slug>` - Deck Permalink
- `https://app.example.com/profile/<handle>/deck/<slug>/card/<rkey>` - Card Permalink
- `https://app.example.com/profile/<handle>/note/<rkey>` - Note Permalink
- `https://app.example.com/profile/<handle>/source/<rkey>` - Source Permalink
