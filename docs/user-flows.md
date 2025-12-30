# User Flows

User experience pathways for Malfestio.

## Authentication

### Login

1. Navigate to `/login`
2. Enter Bluesky handle and app password
3. Submit → redirected to Library

### Logout

1. Click avatar in header → "Logout"
2. → redirected to Landing page

## Deck Management

### Create Deck

1. Library (`/`) → "Create Deck"
2. Fill: title, description, tags
3. Set visibility (Private/Unlisted/Public/SharedWith)
4. Add cards (front/back, optional hints, card type)
5. Submit → deck created, redirected to Library

### View Deck

1. Library → click deck card
2. View title, description, tags, card list
3. Options: Edit, Study, Back to Library

### Study Deck

1. Deck View → "Study Deck"
2. Study session with keyboard controls
3. Grade cards (1-5), view progress
4. Session complete → return to deck

## Note Management

### Create Note

1. Header → "Notes" → "New Note"
2. Fill: title, body (markdown), tags
3. Add wikilinks with `[[Note Title]]`
4. Set visibility
5. Submit → note created

### View Notes

1. Header → "Notes"
2. Browse notes with backlink navigation

## Content Import

### Import Article

1. Header → "Import"
2. Enter article URL
3. Submit → article parsed, deck/note created

### Import Lecture

1. Import page → "Lecture Import" tab
2. Enter lecture URL
3. Submit → lecture content extracted

## Study Session

### Daily Review

1. Navigate to `/review` or click "Review" in header
2. View study stats: due count, streak, reviewed today
3. Click "Start Study Session"
4. Card front shown → press **Space** to flip
5. View answer → grade with **1-5** keys
6. Repeat until all due cards complete
7. View completion message and updated stats

### Deck-Specific Review

1. Navigate to deck view (`/decks/:id`)
2. Click "Study Deck"
3. Review only cards from that deck
4. Same keyboard controls apply

### Progress Tracking

- **Due count**: Cards needing review today
- **Streak**: Consecutive days studied
- **Reviewed today**: Cards completed this session
- **Interval growth**: SM-2 algorithm increases intervals for mastered cards

### Keyboard Shortcuts

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
