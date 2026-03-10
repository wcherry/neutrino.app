# UI State Patterns

## Overview

Every data-fetching surface in Neutrino follows a consistent state machine: **loading → success | error | empty**. This document defines the standard patterns for each state, built on TanStack Query + Zustand.

---

## Data Fetching with TanStack Query

### Query Keys

Use descriptive, hierarchical query key arrays:

```ts
// Files list
['files', { folderId, page, pageSize, sortBy, sortDir }]

// Single file
['files', fileId]

// Quota
['quota', userId]

// Folder contents
['folders', folderId, 'contents']
```

### Standard Query Pattern

```tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { storageApi } from '@/lib/api';
import { FileListSkeleton, EmptyState, Alert } from '@neutrino/ui';

function FileList({ folderId }: { folderId?: string }) {
  const { data, isLoading, isError, error, refetch } = useQuery({
    queryKey: ['files', { folderId, page: 1, pageSize: 20 }],
    queryFn: () => storageApi.listFiles({ page: 1, pageSize: 20 }),
  });

  if (isLoading) {
    return <FileListSkeleton rows={5} />;
  }

  if (isError) {
    return (
      <Alert
        variant="error"
        title="Could not load files"
        message={error instanceof Error ? error.message : 'An unexpected error occurred.'}
        action={<Button size="sm" variant="secondary" onClick={() => refetch()}>Retry</Button>}
      />
    );
  }

  if (data?.items.length === 0) {
    return (
      <EmptyState
        icon={FolderOpen}
        title="No files here"
        description="Upload files to get started."
      />
    );
  }

  return <FileGrid files={data.items} />;
}
```

---

## Loading States

### Rule: Skeleton > Spinner for Content

| Context | Use |
|---|---|
| List/grid content areas | `<FileListSkeleton>` or `<Skeleton>` |
| Full-page initial load | Skeleton layout matching the page |
| Small inline actions (submit button) | `<Button loading>` |
| Overlay (rare, blocking operation) | `<Spinner overlay>` |

### Skeleton Composition

```tsx
// Match the visual shape of what will load
function FileCardSkeleton() {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
      <Skeleton shape="rect" height={100} />  {/* preview area */}
      <Skeleton shape="text" width="80%" height="0.875rem" />
      <Skeleton shape="text" width="50%" height="0.75rem" />
    </div>
  );
}
```

---

## Error States

### Inline vs Full-Page Errors

**Inline (partial failure):** Use `Alert` within the affected content area.

```tsx
{queryError && (
  <Alert
    variant="error"
    title="Could not load recent files"
    message="Check your connection and try again."
    onClose={() => queryError.reset?.()}
  />
)}
```

**Full-page (route-level failure):** Use Next.js `error.tsx` with an `EmptyState`.

```tsx
// app/(drive)/error.tsx
'use client';
export default function DriveError({ reset }: { reset: () => void }) {
  return (
    <div style={{ padding: 'var(--space-16)', textAlign: 'center' }}>
      <EmptyState
        icon={AlertCircle}
        title="Something went wrong"
        description="We couldn't load this page. Please try again."
        action={<Button onClick={reset}>Try again</Button>}
      />
    </div>
  );
}
```

---

## Optimistic Updates

For fast UI feedback on mutations, use TanStack Query's optimistic update pattern:

```tsx
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { storageApi } from '@/lib/api';

function useDeleteFile() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: storageApi.deleteFile,

    // 1. Snapshot current data
    onMutate: async (fileId) => {
      await queryClient.cancelQueries({ queryKey: ['files'] });
      const previous = queryClient.getQueryData(['files', { page: 1, pageSize: 20 }]);

      // 2. Optimistically remove the file from the list
      queryClient.setQueryData(
        ['files', { page: 1, pageSize: 20 }],
        (old: any) => ({
          ...old,
          items: old.items.filter((f: any) => f.id !== fileId),
          total: old.total - 1,
        })
      );

      return { previous };
    },

    // 3. Roll back on error
    onError: (_err, _fileId, context) => {
      queryClient.setQueryData(['files', { page: 1, pageSize: 20 }], context?.previous);
      toast.error('Could not delete file. Please try again.');
    },

    // 4. Refetch to sync with server
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['files'] });
    },
  });
}
```

---

## Upload Progress State

```tsx
'use client';

import { useState } from 'react';
import { storageApi } from '@/lib/api';
import { ProgressBar, Button } from '@neutrino/ui';

function FileUploader() {
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setError(null);
    setProgress(0);

    try {
      await storageApi.uploadFile(file, setProgress);
      setProgress(null);
      toast.success(`${file.name} uploaded successfully`);
    } catch (err) {
      setProgress(null);
      setError(err instanceof Error ? err.message : 'Upload failed');
    }
  };

  return (
    <div>
      <input type="file" onChange={handleFileChange} />
      {progress !== null && (
        <ProgressBar
          value={progress}
          max={100}
          label="Uploading..."
          showValue
          color={progress === 100 ? 'success' : 'accent'}
        />
      )}
      {error && <Alert variant="error" message={error} />}
    </div>
  );
}
```

---

## Empty States

Provide clear, actionable empty states for every list surface.

### Pattern

1. **Contextual icon** — represents the content type
2. **Specific title** — what is empty (not just "Nothing here")
3. **Helpful description** — why it's empty and what to do
4. **Primary action** — the next logical step

```tsx
// Good
<EmptyState
  icon={FolderOpen}
  title="This folder is empty"
  description="Drag files here or click Upload to add files."
  action={<Button icon={<Upload size={16} />}>Upload files</Button>}
/>

// Bad
<EmptyState title="Nothing here" />
```

### Variations by context

| Context | Title | CTA |
|---|---|---|
| All files (no uploads yet) | "No files yet" | Upload first file |
| Search with no results | `No results for "${query}"` | Clear search |
| Starred (nothing starred) | "No starred files" | Star a file |
| Trash (empty trash) | "Trash is empty" | None |
| Shared with me (none shared) | "No files shared with you" | None |

---

## Zustand for Global State

Use Zustand for client-side global state that doesn't belong in TanStack Query (e.g., UI state, user preferences, upload queue).

```ts
// lib/store/uploadStore.ts
import { create } from 'zustand';

interface UploadItem {
  id: string;
  name: string;
  progress: number;
  status: 'pending' | 'uploading' | 'done' | 'error';
  error?: string;
}

interface UploadStore {
  queue: UploadItem[];
  addToQueue: (item: UploadItem) => void;
  updateProgress: (id: string, progress: number) => void;
  markDone: (id: string) => void;
  markError: (id: string, error: string) => void;
  remove: (id: string) => void;
  clearCompleted: () => void;
}

export const useUploadStore = create<UploadStore>((set) => ({
  queue: [],
  addToQueue: (item) => set((s) => ({ queue: [...s.queue, item] })),
  updateProgress: (id, progress) =>
    set((s) => ({
      queue: s.queue.map((u) => (u.id === id ? { ...u, progress } : u)),
    })),
  markDone: (id) =>
    set((s) => ({
      queue: s.queue.map((u) => (u.id === id ? { ...u, status: 'done', progress: 100 } : u)),
    })),
  markError: (id, error) =>
    set((s) => ({
      queue: s.queue.map((u) => (u.id === id ? { ...u, status: 'error', error } : u)),
    })),
  remove: (id) => set((s) => ({ queue: s.queue.filter((u) => u.id !== id) })),
  clearCompleted: () =>
    set((s) => ({ queue: s.queue.filter((u) => u.status !== 'done') })),
}));
```

---

## Form State with React Hook Form + Zod

```tsx
'use client';

import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { TextInput, Button } from '@neutrino/ui';

const loginSchema = z.object({
  email: z.string().email('Please enter a valid email'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
});

type LoginForm = z.infer<typeof loginSchema>;

function LoginForm() {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<LoginForm>({ resolver: zodResolver(loginSchema) });

  const onSubmit = async (data: LoginForm) => {
    await authApi.login(data);
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <TextInput
        label="Email"
        type="email"
        error={errors.email?.message}
        {...register('email')}
      />
      <TextInput
        label="Password"
        type="password"
        error={errors.password?.message}
        {...register('password')}
      />
      <Button type="submit" loading={isSubmitting}>
        Sign in
      </Button>
    </form>
  );
}
```
