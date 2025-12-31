# SolidJS UI Component Library

Carbon-inspired component library for Malfestio.

## Animations

Presets defined in `src/lib/animations.ts`:

- `fadeIn`, `fadeOut` - opacity transitions
- `slideInRight`, `slideInUp` - directional entry
- `scaleIn`, `scaleOut` - scale with opacity
- `springConfig` - natural bounce settings
- `staggerDelay(index)` - list animation helper

## Components

### Core

| Component  | File                | Purpose                                          |
| ---------- | ------------------- | ------------------------------------------------ |
| Dialog     | `ui/Dialog.tsx`     | Modal with transactional/danger/passive variants |
| TreeView   | `ui/TreeView.tsx`   | Expandable hierarchy with keyboard nav           |
| DataTable  | `ui/DataTable.tsx`  | Sortable, selectable, expandable rows            |
| SidePanel  | `ui/SidePanel.tsx`  | Collapsible left navigation                      |
| RightPanel | `ui/RightPanel.tsx` | Slide-in detail panel                            |
| Tabs       | `ui/Tabs.tsx`       | Line and contained variants                      |
| Dropdown   | `ui/Dropdown.tsx`   | Single/multi select with search                  |
| Tag        | `ui/Tag.tsx`        | Read-only, dismissible, selectable               |

### Supporting

| Component   | File                 | Purpose                      |
| ----------- | -------------------- | ---------------------------- |
| Skeleton    | `ui/Skeleton.tsx`    | Loading placeholders         |
| EmptyState  | `ui/EmptyState.tsx`  | Zero-state UI                |
| Tooltip     | `ui/Tooltip.tsx`     | Hover info with positioning  |
| ProgressBar | `ui/ProgressBar.tsx` | Determinate/indeterminate    |
| Avatar      | `ui/Avatar.tsx`      | Image with initials fallback |
| Menu        | `ui/Menu.tsx`        | Context/dropdown actions     |

### Existing

| Component | File            | Purpose                        |
| --------- | --------------- | ------------------------------ |
| Button    | `ui/Button.tsx` | Primary/secondary/danger/ghost |
| Card      | `ui/Card.tsx`   | Container with optional title  |
| Toast     | `ui/Toast.tsx`  | Notifications                  |

## Keyboard Navigation

- **Dialog**: `Escape` to close
- **TreeView**: Arrow keys, `Enter`/`Space` to expand
- **Tabs**: Left/Right arrows
- **Dropdown**: Arrow keys, `Enter` to select, `Escape` to close
- **Menu**: Arrow keys, `Enter` to select

## Common Gotchas

### Router Components in Module-Level Constants

Router components (`<A>`, and hooks like `useNavigate`, `useParams`) must be created during component render, not at module initialization.

**Problem:**

```tsx
// ✗ JSX created when module loads, before Router exists
const config = {
  action: <A href="/somewhere">Click me</A>
}
```

**Solution:**

```tsx
// ✓ JSX created during render, inside Router context
const getConfig = () => ({
  action: () => <A href="/somewhere">Click me</A>
})

const MyComponent = () => {
  const config = getConfig();
  return <div>{config.action()}</div>
}
```

**Why:** Module-level JSX executes immediately on import, before `<Router>` establishes its context. Router components need that context to function.

**Rule:** If storing JSX that contains router components, wrap it in a function to defer creation until render time.
