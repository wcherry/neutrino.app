# Design Tokens

All design tokens are CSS custom properties defined in `packages/ui/src/tokens/`. They are imported automatically when you import `@neutrino/ui/styles`.

## Color Tokens

### Neutrals
| Token | Value | Usage |
|---|---|---|
| `--color-neutral-50` | `#f9fafb` | Background subtle |
| `--color-neutral-100` | `#f3f4f6` | Surface raised |
| `--color-neutral-200` | `#e5e7eb` | Borders |
| `--color-neutral-300` | `#d1d5db` | Strong borders |
| `--color-neutral-400` | `#9ca3af` | Muted text |
| `--color-neutral-500` | `#6b7280` | Secondary text |
| `--color-neutral-600` | `#4b5563` | Secondary text |
| `--color-neutral-700` | `#374151` | |
| `--color-neutral-800` | `#1f2937` | |
| `--color-neutral-900` | `#111827` | Primary text |

### Semantic Color Aliases

These are what you should use in components — they automatically switch in dark mode.

| Token | Light Value | Dark Value | Usage |
|---|---|---|---|
| `--color-bg` | `#ffffff` | `#0f172a` | Page background |
| `--color-bg-subtle` | neutral-50 | `#1e293b` | Sidebar, code blocks |
| `--color-surface` | `#ffffff` | `#1e293b` | Cards, panels |
| `--color-surface-raised` | neutral-50 | `#334155` | Dropdown, tooltips |
| `--color-surface-overlay` | `#ffffff` | `#1e293b` | Modals, overlays |
| `--color-border` | neutral-200 | `#334155` | Default borders |
| `--color-border-strong` | neutral-300 | `#475569` | Input focus borders |

### Text Colors
| Token | Usage |
|---|---|
| `--color-text-primary` | Body text, headings |
| `--color-text-secondary` | Supporting text |
| `--color-text-muted` | Placeholder, captions |
| `--color-text-inverse` | Text on dark backgrounds |
| `--color-text-on-accent` | Text on accent-colored elements |

### Accent (Drive Blue)
| Token | Light | Dark |
|---|---|---|
| `--color-accent` | `#2563eb` | `#3b82f6` |
| `--color-accent-hover` | `#1d4ed8` | `#60a5fa` |
| `--color-accent-subtle` | `#eff6ff` | `#1e3a5f` |
| `--color-accent-text` | `#1e40af` | `#93c5fd` |

### Status Colors
| Token | Value | Subtle |
|---|---|---|
| `--color-success` | `#16a34a` | `--color-success-subtle: #f0fdf4` |
| `--color-warning` | `#d97706` | `--color-warning-subtle: #fffbeb` |
| `--color-error` | `#dc2626` | `--color-error-subtle: #fef2f2` |
| `--color-info` | `#0284c7` | `--color-info-subtle: #f0f9ff` |

---

## Spacing Tokens

| Token | Value | px equivalent |
|---|---|---|
| `--space-0` | `0` | 0 |
| `--space-1` | `0.25rem` | 4px |
| `--space-2` | `0.5rem` | 8px |
| `--space-3` | `0.75rem` | 12px |
| `--space-4` | `1rem` | 16px |
| `--space-5` | `1.25rem` | 20px |
| `--space-6` | `1.5rem` | 24px |
| `--space-8` | `2rem` | 32px |
| `--space-10` | `2.5rem` | 40px |
| `--space-12` | `3rem` | 48px |
| `--space-16` | `4rem` | 64px |
| `--space-20` | `5rem` | 80px |
| `--space-24` | `6rem` | 96px |

### Border Radius
| Token | Value |
|---|---|
| `--radius-sm` | `0.25rem` (4px) |
| `--radius-md` | `0.375rem` (6px) |
| `--radius-lg` | `0.5rem` (8px) |
| `--radius-xl` | `0.75rem` (12px) |
| `--radius-full` | `9999px` |

---

## Typography Tokens

### Font Families
| Token | Value |
|---|---|
| `--font-sans` | `'Inter', system-ui, -apple-system, sans-serif` |
| `--font-mono` | `'JetBrains Mono', 'Fira Code', monospace` |

### Font Sizes
| Token | Value | px |
|---|---|---|
| `--text-xs` | `0.75rem` | 12px |
| `--text-sm` | `0.875rem` | 14px |
| `--text-base` | `1rem` | 16px |
| `--text-lg` | `1.125rem` | 18px |
| `--text-xl` | `1.25rem` | 20px |
| `--text-2xl` | `1.5rem` | 24px |
| `--text-3xl` | `1.875rem` | 30px |
| `--text-4xl` | `2.25rem` | 36px |

### Font Weights
| Token | Value |
|---|---|
| `--font-normal` | `400` |
| `--font-medium` | `500` |
| `--font-semibold` | `600` |
| `--font-bold` | `700` |

### Line Heights
| Token | Value |
|---|---|
| `--leading-tight` | `1.25` |
| `--leading-normal` | `1.5` |
| `--leading-relaxed` | `1.75` |

---

## Shadow Tokens

| Token | Usage |
|---|---|
| `--shadow-sm` | Subtle elevation for inputs, cards at rest |
| `--shadow-md` | Cards on hover, dropdowns |
| `--shadow-lg` | Modals, drawers, toasts |
| `--shadow-xl` | High-emphasis overlays |

---

## Z-Index Tokens

| Token | Value | Usage |
|---|---|---|
| `--z-base` | `0` | Normal content |
| `--z-dropdown` | `100` | Dropdowns, menus |
| `--z-sticky` | `200` | Sticky headers |
| `--z-modal-backdrop` | `300` | Modal backdrop |
| `--z-modal` | `400` | Modal content |
| `--z-popover` | `500` | Tooltips, popovers |
| `--z-toast` | `600` | Toasts (always on top) |

---

## Motion Tokens

| Token | Value | Usage |
|---|---|---|
| `--duration-instant` | `50ms` | Micro interactions |
| `--duration-fast` | `100ms` | Hover states, color transitions |
| `--duration-normal` | `200ms` | Most transitions |
| `--duration-slow` | `300ms` | Expanding panels |
| `--duration-slower` | `500ms` | Page transitions |
| `--ease-default` | `cubic-bezier(0.4, 0, 0.2, 1)` | General purpose |
| `--ease-in` | `cubic-bezier(0.4, 0, 1, 1)` | Elements leaving screen |
| `--ease-out` | `cubic-bezier(0, 0, 0.2, 1)` | Elements entering screen |
| `--ease-spring` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | Bouncy / playful |

All motion respects `prefers-reduced-motion: reduce` — durations collapse to `0.01ms`.

---

## Dark Mode

Dark mode is activated by setting `data-theme="dark"` on any ancestor element. The root `<html>` element is the standard target.

```html
<!-- Light mode (default) -->
<html lang="en" data-app="drive">

<!-- Dark mode -->
<html lang="en" data-app="drive" data-theme="dark">
```

Dark mode is implemented entirely in CSS — no JavaScript token overrides.

---

## Using Tokens in CSS Modules

Always reference tokens, never hard-code values:

```css
/* Good */
.card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
}

/* Bad */
.card {
  background-color: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 16px;
}
```
