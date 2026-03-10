# Performance Guide

## Bundle Size Targets

| Bundle | Target | Measurement |
|---|---|---|
| `@neutrino/ui` tree-shaken import | < 50 KB gzipped | `next build` output |
| Initial JS per route | < 100 KB gzipped | Lighthouse / `next build` |
| Total CSS | < 20 KB gzipped | `next build` output |
| Largest Contentful Paint (LCP) | < 2.5s | Lighthouse |
| Total Blocking Time (TBT) | < 200ms | Lighthouse |
| Cumulative Layout Shift (CLS) | < 0.1 | Lighthouse |

---

## Tree Shaking

The component library uses named exports — only import what you use:

```tsx
// Good: tree-shaken — only Button and Card are bundled
import { Button, Card } from '@neutrino/ui';

// Bad: barrel import with side effects — may pull in everything
import * as UI from '@neutrino/ui';
```

---

## Code Splitting

Next.js App Router handles route-level code splitting automatically. For heavy components not needed on initial load, use dynamic imports:

```tsx
import dynamic from 'next/dynamic';

// The rich text editor only loads when the user opens the editor
const RichEditor = dynamic(
  () => import('@/components/RichEditor'),
  {
    loading: () => <Skeleton height={200} />,
    ssr: false,
  }
);
```

Components that should always be lazy-loaded:
- PDF viewers
- Chart libraries
- Video players
- Map components
- Image editors

---

## CSS-over-JS Animations

Prefer CSS transitions for hover/focus states — they don't require JavaScript and run on the compositor thread.

```css
/* Good: CSS transition */
.button {
  transition: background-color var(--duration-fast) var(--ease-default);
}

.button:hover {
  background-color: var(--color-accent-hover);
}
```

Only use Framer Motion for:
- Page/route transitions
- List item enter/exit (AnimatePresence)
- Modals, drawers, and overlays
- Complex layout animations

Avoid Framer Motion for:
- Hover effects
- Focus rings
- Color changes
- Simple opacity toggles

---

## Image Optimization

Always use Next.js `<Image>` for user-uploaded content thumbnails and any image that could be large:

```tsx
import Image from 'next/image';

<Image
  src={file.thumbnailUrl}
  alt={file.name}
  width={160}
  height={120}
  loading="lazy"
  placeholder="blur"
  blurDataURL={file.blurHash}
/>
```

For icons, always use Lucide React (SVG, inline) — never PNG/JPG icons.

---

## React Server Components (RSC)

Maximize server component usage to reduce client-side JavaScript:

**Use Server Components (default) for:**
- Static layouts (AppShell structure, page headings)
- Data fetching with `fetch()` or ORM calls
- Components that only read data and render HTML

**Use Client Components (`'use client'`) only for:**
- Interactive event handlers (`onClick`, `onChange`)
- Browser APIs (`localStorage`, `window`, `document`)
- React hooks (`useState`, `useEffect`, `useContext`)
- Framer Motion animations (requires browser)

```tsx
// Server Component — no 'use client'
// Fetches data directly, no hydration overhead
async function FilePage({ params }: { params: { id: string } }) {
  const file = await storageApi.getFileMetadata(params.id); // server-side fetch
  return <FileDetail file={file} />;
}

// Client Component — only for the interactive part
'use client';
function FileActions({ fileId }: { fileId: string }) {
  const [menuOpen, setMenuOpen] = useState(false);
  return <Dropdown open={menuOpen} onClose={() => setMenuOpen(false)} ... />;
}
```

---

## TanStack Query Optimization

### Stale Time Configuration

```ts
// In QueryProvider
new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60,      // 1 minute: don't refetch unless stale
      gcTime: 1000 * 60 * 5,     // 5 minutes: keep in cache
    },
  },
});

// Per-query overrides
useQuery({
  queryKey: ['quota'],
  queryFn: storageApi.getQuota,
  staleTime: 1000 * 30,          // Quota changes frequently, refresh every 30s
});

useQuery({
  queryKey: ['files', fileId, 'metadata'],
  queryFn: () => storageApi.getFileMetadata(fileId),
  staleTime: Infinity,            // File metadata rarely changes
});
```

### Prefetching

Prefetch data before users navigate to improve perceived performance:

```tsx
const queryClient = useQueryClient();

// Prefetch file details on hover
<FileCard
  onMouseEnter={() => {
    queryClient.prefetchQuery({
      queryKey: ['files', file.id],
      queryFn: () => storageApi.getFileMetadata(file.id),
    });
  }}
/>
```

---

## Pagination vs. Infinite Scroll

| Context | Strategy | Reason |
|---|---|---|
| File/folder grid | Pagination | Deep linking, browser back, shareable URLs |
| Search results | Pagination | Deterministic results, shareable |
| Activity feed | Infinite scroll | Chronological, no need to go to page 5 |
| Notifications | Infinite scroll | Same as above |

Use the `<Pagination>` component for paginated views — it renders proper `<nav>` with ARIA.

---

## Layout Shift Prevention

Always define dimensions for images and loading skeletons to prevent CLS:

```tsx
// Good: explicit dimensions prevent layout shift
<Skeleton width={160} height={120} shape="rect" />

// Good: image with explicit dimensions
<Image width={160} height={120} src={url} alt={name} />

// Bad: content pops in and causes layout shift
{isLoading ? null : <FileCard file={file} />}
```

---

## Monitoring

In production, instrument with:

1. **Web Vitals** via Next.js built-in reporting
2. **Sentry** for JS error tracking
3. **OpenTelemetry** traces from Next.js server to Rust backend
4. **Lighthouse CI** in CI pipeline — fail build if score drops below threshold

```ts
// app/layout.tsx — report Web Vitals
export function reportWebVitals(metric: NextWebVitalsMetric) {
  console.log(metric); // Replace with your analytics endpoint
}
```
