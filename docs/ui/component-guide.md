# Component Guide

## Overview

The Neutrino UI library (`@neutrino/ui`) is a custom component system built with React, TypeScript, and CSS Modules. It uses CSS custom properties (design tokens) exclusively — no Tailwind, no inline styles for token values.

## Installation & Setup

```ts
// In your Next.js layout
import '@neutrino/ui/styles'; // imports all CSS tokens + global styles

// Import components
import { Button, Card, Modal } from '@neutrino/ui';
```

## Component Categories

### Primitives
Low-level building blocks: `Button`, `Text`, `Heading`, `Link`, `Divider`, `Badge`, `Avatar`

### Inputs
Form controls: `TextInput`, `Textarea`, `Select`, `Checkbox`, `Radio`, `RadioGroup`, `Toggle`, `SearchInput`

### Feedback
Status communication: `Alert`, `Toast`, `ToastProvider`, `ProgressBar`, `Spinner`, `Skeleton`, `FileListSkeleton`, `EmptyState`

### Containers
Layout shells: `Card`, `CardHeader`, `CardFooter`, `Panel`, `Modal`, `ModalHeader`, `ModalBody`, `ModalFooter`, `Popover`, `Drawer`, `Tabs`, `TabList`, `Tab`, `TabPanel`, `Accordion`, `AccordionItem`

### Navigation
Navigation patterns: `Breadcrumbs`, `Pagination`, `Menu`, `MenuItem`, `MenuSeparator`, `MenuGroup`, `Dropdown`

### Shell
App-level layout: `AppShell`, `Sidebar`, `Topbar`

---

## Button

```tsx
import { Button } from '@neutrino/ui';

// Variants
<Button variant="primary">Save</Button>
<Button variant="secondary">Cancel</Button>
<Button variant="ghost">Dismiss</Button>
<Button variant="danger">Delete</Button>

// Sizes
<Button size="sm">Small</Button>
<Button size="md">Medium</Button>  // default
<Button size="lg">Large</Button>

// With icon
import { Upload } from 'lucide-react';
<Button icon={<Upload size={16} />}>Upload</Button>
<Button icon={<Upload size={16} />} iconPosition="right">Upload</Button>

// Loading state
<Button loading>Saving...</Button>

// Disabled
<Button disabled>Cannot click</Button>
```

**Do:**
- Use `primary` for the main call-to-action on a page/section
- Use `secondary` for secondary actions that sit alongside a primary
- Use `ghost` for tertiary actions and icon-only toolbar buttons
- Use `danger` only for destructive actions (delete, revoke access)
- Always provide accessible labels for icon-only buttons

**Don't:**
- Don't use more than one `primary` button per section
- Don't use `danger` to indicate error state — use `Alert` instead
- Don't nest buttons

---

## TextInput

```tsx
import { TextInput } from '@neutrino/ui';
import { Mail } from 'lucide-react';

<TextInput
  label="Email address"
  placeholder="you@example.com"
  type="email"
  required
  hint="We'll never share your email"
  iconLeft={<Mail size={16} />}
/>

// Error state
<TextInput
  label="Password"
  type="password"
  error="Password must be at least 8 characters"
/>
```

---

## Toast / Notifications

```tsx
import { ToastProvider, useToast } from '@neutrino/ui';

// Wrap your app
<ToastProvider position="bottom-right">
  <App />
</ToastProvider>

// In any client component
function MyComponent() {
  const toast = useToast();

  return (
    <Button onClick={() => toast.success('File uploaded!', 'Success')}>
      Upload
    </Button>
  );
}
```

Toast variants: `success`, `error`, `warning`, `info`

---

## Modal

```tsx
import { Modal, ModalHeader, ModalBody, ModalFooter, Button } from '@neutrino/ui';

function DeleteDialog({ open, onClose, onConfirm }) {
  return (
    <Modal open={open} onClose={onClose} size="sm">
      <ModalHeader title="Delete file?" onClose={onClose} />
      <ModalBody>
        <Text>This action cannot be undone. The file will be permanently deleted.</Text>
      </ModalBody>
      <ModalFooter>
        <Button variant="secondary" onClick={onClose}>Cancel</Button>
        <Button variant="danger" onClick={onConfirm}>Delete</Button>
      </ModalFooter>
    </Modal>
  );
}
```

Modal includes a focus trap — keyboard users are contained within the modal when it's open. Press Escape to close.

---

## Tabs

```tsx
import { Tabs, TabList, Tab, TabPanel } from '@neutrino/ui';

<Tabs defaultTab="files">
  <TabList>
    <Tab id="files" badge={12}>Files</Tab>
    <Tab id="shared">Shared</Tab>
    <Tab id="trash" icon={<Trash2 size={14} />}>Trash</Tab>
  </TabList>
  <TabPanel id="files">
    {/* file list */}
  </TabPanel>
  <TabPanel id="shared">
    {/* shared files */}
  </TabPanel>
  <TabPanel id="trash">
    {/* trash */}
  </TabPanel>
</Tabs>
```

Controlled mode:

```tsx
const [tab, setTab] = useState('files');
<Tabs value={tab} onChange={setTab}>...</Tabs>
```

---

## AppShell

```tsx
import { AppShell, Sidebar, Topbar } from '@neutrino/ui';

<AppShell
  sidebar={<Sidebar sections={navSections} quota={quota} onUpload={handleUpload} />}
  topbar={<Topbar user={currentUser} onSearch={handleSearch} onSignOut={handleSignOut} />}
>
  {/* page content */}
</AppShell>
```

The shell is CSS Grid-based: `240px sidebar | 1fr content`, `64px topbar | 1fr main`. On mobile (≤768px) the sidebar is hidden and toggled via a menu button in the Topbar.

---

## Accordion

```tsx
import { Accordion, AccordionItem } from '@neutrino/ui';

<Accordion defaultOpen="faq-1">
  <AccordionItem id="faq-1" title="How does storage work?">
    Files are stored on our servers and accessible from any device.
  </AccordionItem>
  <AccordionItem id="faq-2" title="What file types are supported?">
    All file types up to 10 GB per file.
  </AccordionItem>
</Accordion>

// Multiple open at once
<Accordion multiple>...</Accordion>
```

---

## Skeleton Loading

```tsx
import { Skeleton, FileListSkeleton } from '@neutrino/ui';

// Generic skeleton
<Skeleton width="60%" height="1rem" shape="text" />
<Skeleton width={48} height={48} shape="circle" />

// Pre-built file list skeleton
<FileListSkeleton rows={5} />
```

Always show skeletons rather than spinners for content areas to reduce perceived loading time.

---

## EmptyState

```tsx
import { EmptyState } from '@neutrino/ui';
import { FolderOpen, Upload } from 'lucide-react';

<EmptyState
  icon={FolderOpen}
  title="No files yet"
  description="Upload files to get started. All your files will appear here."
  action={
    <Button variant="primary" icon={<Upload size={16} />}>
      Upload files
    </Button>
  }
/>
```
