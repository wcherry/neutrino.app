# Accessibility Standards

## Target Compliance

Neutrino UI targets **WCAG 2.1 AA** compliance across all components.

---

## Core Principles

### 1. Semantic HTML First

Use the correct HTML element before reaching for ARIA. Screen readers understand native semantics best.

```tsx
// Good: native button semantics
<button type="button" onClick={handler}>Upload</button>

// Bad: div with click handler
<div role="button" onClick={handler}>Upload</div>
```

Components like `Button`, `TextInput`, `Checkbox`, `Radio`, and `Toggle` use their native HTML counterparts under the hood.

### 2. Keyboard Navigation

Every interactive element must be reachable and operable with the keyboard alone.

| Key | Behavior |
|---|---|
| `Tab` | Move to next focusable element |
| `Shift+Tab` | Move to previous focusable element |
| `Enter` / `Space` | Activate buttons, checkboxes |
| `Arrow keys` | Navigate within radio groups, tabs, menus |
| `Escape` | Close modals, drawers, dropdowns |
| `Home` / `End` | Jump to first/last item in a list |

### 3. Focus Management

Focus must be managed programmatically when UI changes:

- **Modals**: On open, focus moves to first focusable element. A focus trap keeps Tab cycles inside. On close, focus returns to the trigger element.
- **Dropdowns/Menus**: On open, focus moves to first item. Arrow keys navigate items. Escape closes and returns focus to trigger.
- **Toast notifications**: Non-interactive — announced via `aria-live` regions.

### 4. Color Contrast

All text must meet WCAG AA contrast ratios:

| Text type | Minimum contrast |
|---|---|
| Normal text (< 18pt) | 4.5 : 1 |
| Large text (≥ 18pt / 14pt bold) | 3 : 1 |
| UI components, graphical objects | 3 : 1 |

The color tokens have been chosen to meet these ratios in both light and dark modes.

**Never** convey information with color alone. Always pair color with:
- Text labels
- Icons
- Patterns or shapes

---

## ARIA Attributes

### Descriptive Labels

```tsx
// Use aria-label for icon-only buttons
<button aria-label="Upload files">
  <Upload size={18} />
</button>

// Use aria-labelledby for complex inputs
<label id="search-label">Search files</label>
<input aria-labelledby="search-label" />

// Use aria-describedby for hints/errors
<TextInput
  label="Password"
  hint="At least 8 characters"
  error="Too short"
/>
// ^ automatically wires aria-describedby
```

### Status & Live Regions

```tsx
// Announce dynamic content changes
<div aria-live="polite">
  {isLoading ? 'Loading files...' : `${total} files found`}
</div>

// Assertive for errors
<div role="alert" aria-live="assertive">
  {error}
</div>
```

Our `Alert`, `Toast`, and form error messages use the correct live region roles automatically.

### Interactive Widgets

| Widget | Required ARIA |
|---|---|
| Toggle/Switch | `role="switch"` + `aria-checked` |
| Checkbox (indeterminate) | `aria-checked="mixed"` |
| Tabs | `role="tablist"`, `role="tab"`, `role="tabpanel"`, `aria-selected`, `aria-controls` |
| Modal | `role="dialog"`, `aria-modal="true"`, `aria-labelledby` |
| Dropdown menu | `aria-haspopup="true"`, `aria-expanded`, `role="menu"`, `role="menuitem"` |
| Progress bar | `role="progressbar"`, `aria-valuenow`, `aria-valuemin`, `aria-valuemax` |
| Breadcrumb | `<nav aria-label="Breadcrumb">`, `aria-current="page"` on last item |

---

## Screen Reader Patterns

### Visually Hidden Text

Use the `.u-sr-only` utility class to provide context to screen readers without visual impact:

```tsx
<button>
  <Upload size={18} aria-hidden="true" />
  <span className="u-sr-only">Upload files</span>
</button>
```

### Loading States

```tsx
// Skeleton loaders should have a role and label
<div role="status" aria-label="Loading files...">
  <FileListSkeleton />
</div>

// Spinner
<Spinner label="Uploading your file..." />
```

### Empty States

```tsx
<EmptyState
  title="No files yet"
  description="Upload files to get started"
  // role="status" is applied automatically
/>
```

---

## Motion & Animation

Respect user preferences for reduced motion:

```css
/* Already included in tokens/motion.css */
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    transition-duration: 0.01ms !important;
    animation-duration: 0.01ms !important;
  }
}
```

For Framer Motion animations, always check `useReducedMotion()`:

```tsx
import { useReducedMotion } from 'framer-motion';

function AnimatedComponent() {
  const shouldReduce = useReducedMotion();
  return (
    <motion.div
      animate={{ opacity: 1, y: shouldReduce ? 0 : 0 }}
      initial={{ opacity: 0, y: shouldReduce ? 0 : 8 }}
    >
      ...
    </motion.div>
  );
}
```

---

## Testing Checklist

Before shipping a component or page:

- [ ] All interactive elements are reachable via Tab
- [ ] All interactive elements have visible focus indicators
- [ ] Color contrast meets 4.5:1 for text
- [ ] No information is conveyed by color alone
- [ ] All images have `alt` text
- [ ] All form inputs have associated labels
- [ ] Form errors are announced to screen readers
- [ ] Modals and drawers trap focus when open
- [ ] Modals close on Escape and return focus to trigger
- [ ] Dynamic content changes are announced via live regions
- [ ] Icon-only buttons have `aria-label`
- [ ] Components work with keyboard only (no mouse)
- [ ] Reduced motion preference is respected

---

## Tools

- **Axe DevTools** browser extension — automated a11y scanning
- **NVDA** (Windows) / **VoiceOver** (Mac) — screen reader testing
- **Colour Contrast Analyser** — verify contrast ratios
- **Playwright** — automated a11y checks in E2E tests using `@axe-core/playwright`
