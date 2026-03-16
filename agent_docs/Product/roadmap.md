# Neutrino – Product Roadmap

A phased roadmap for building a Google Drive-competitive cloud storage and collaboration platform. Each phase builds on the previous one, prioritizing a shippable core before expanding into collaboration, intelligence, and enterprise features.

---

## Phase 1 — Core Storage & File Management (MVP)

**Goal:** Users can store, organize, and access files from a web browser. A working, reliable foundation.

### 1.0 Boot strapping
- [x] Create cargo workspaces and initialization
- [x] Create drive(api), shared, and worker projects
- [x] Add healthcheck - checks DB for now
- [x] Implement Authentication & Authuerization

### 1.1 Cloud Storage Engine
- [x] Design storage backend (object storage layer, e.g. S3-compatible)
- [x] Implement file upload with multipart support (files up to 10 GB)
- [x] Implement file download with resume support
- [x] Enforce per-user storage quotas
- [x] Track per-user daily upload cap
- [x] Store and retrieve file metadata (name, size, type, owner, created/modified timestamps)

### 1.2 File System & Organization
- [x] Data model: files, folders, nested folder hierarchy
- [x] Create, rename, and delete files and folders
- [x] Move files between folders
- [ ] Drag-and-drop file organization (web UI)
- [x] Star/favorite files for quick access
- [x] Color-label folders
- [x] Shortcuts — link a file into multiple folders without duplication
- [x] Bulk select: move, delete, download multiple items at once
- [x] Trash with 30-day retention and restore

### 1.3 File Preview
- [x] In-browser viewer for PDF
- [x] In-browser viewer for images (JPEG, PNG, GIF, WebP)
- [x] In-browser viewer for video (MP4, MOV)
- [x] In-browser viewer for plain text and code files (syntax highlighting)
- [x] In-browser viewer for ZIP archive (show contents)
- [x] Preview without requiring download

### 1.4 File Versioning
- [x] Automatic version snapshots on every upload/edit
- [x] View version history per file
- [x] Restore a previous version
- [x] Name/label specific versions
- [x] Retain up to 100 versions for non-native files

### 1.5 Web Application (Core UI)
- [x] UI Foundation: pnpm monorepo (apps/web, packages/ui)
- [x] Design tokens: colors, spacing, typography, shadows, motion (CSS custom properties)
- [x] Component library: 30+ components with CSS Modules and full TypeScript types
- [x] AppShell: CSS Grid layout (240px sidebar + 64px topbar + main)
- [x] Drive home page: list/grid view of files with quick access section
- [x] Typed API client for all backend endpoints (auth, storage, filesystem)
- [x] TanStack Query + Zustand + React Hook Form + Zod integration
- [x] WCAG AA accessibility (focus traps, ARIA, live regions, keyboard nav)
- [x] Authentication pages (sign up, sign in, sign out)
- [x] Sort by name, date modified, owner, size
- [x] Filter by file type and starred/shared
- [x] File info panel (metadata, activity, version count)
- [x] Upload via browser file picker and drag-and-drop
- [x] Context menu: rename, move, download, delete, star, copy link

---

## Phase 2 — Sharing & Permissions

**Goal:** Users can share files and folders with fine-grained access control.

### 2.1 Permission Model
- [x] Define roles: Owner, Editor, Commenter, Viewer
- [x] Assign roles per file and per folder
- [x] Folder-level permission inheritance (children inherit parent permissions)
- [x] Revoke access at any time
- [x] Transfer ownership between users

### 2.2 Link Sharing
- [x] Generate shareable links per file/folder
- [x] Set link visibility: public, anyone with link, specific people
- [x] Set link expiration dates
- [x] Disable link sharing (make access-list-only)

### 2.3 Information Rights Management (IRM)
- [x] Restrict download for Viewer and Commenter roles
- [x] Restrict print and copy for Viewer and Commenter roles
- [x] Extend download/print/copy restriction to Editor role (admin-controlled)

### 2.4 Sharing UI
- [x] Share dialog: add people by email, set role
- [x] Show current collaborators and their roles in share dialog
- [x] "Shared with me" section in Drive home
- [x] Email notification to recipient on new share
- [x] Access request flow: recipient requests access, owner approves/denies
- [x] Owner receives push/email notification for access requests

### 2.5 Domain Restrictions (Workspace)
- [x] Restrict shares to within an organization domain
- [x] Block external share from admin console
- [x] Domain-only link sharing option

---

## Phase 3 — Native Editors (Docs, Sheets, Slides)

**Goal:** Users can create and edit documents, spreadsheets, and presentations natively inside Neutrino.

### 3.1 Docs (Word Processor)
- [x] Rich text editor: bold, italic, underline, headings, fonts, font sizes, colors
- [x] Lists (bulleted, numbered, nested)
- [x] Tables (insert, resize, merge cells)
- [x] Images and media insertion
- [x] Headers, footers, and footnotes
- [x] Page setup: margins, orientation, page size
- [x] Columns layout
- [x] Document outline (heading-based navigation)
- [x] Table of contents auto-generation
- [x] Spell check and grammar suggestions
- [x] Word count
- [x] Export: DOCX, PDF, ODT, TXT, HTML
- [x] Import: DOCX, RTF, ODT (up to 50 MB for conversion)
- [x] Document size limit: ~1.02 million characters

### 3.2 Sheets (Spreadsheet)
- [x] Spreadsheet grid: rows, columns, merged cells
- [x] Freeze rows/columns (panes)
- [x] Formula engine: 400+ built-in functions (SUM, VLOOKUP, IF, INDEX/MATCH, etc.)
- [x] Cell formatting: number, currency, date, percentage, custom
- [x] Sort and filter (filter views per user)
- [x] Conditional formatting (color scales, rules)
- [x] Data validation (dropdowns, numeric rules)
- [x] Pivot tables
- [x] Charts and graphs (line, bar, pie, scatter, area)
- [x] Import CSV, TSV; open XLSX without conversion
- [x] Export: XLSX, CSV, PDF
- [x] Cell size limit: 10 million cells per spreadsheet

### 3.3 Slides (Presentation)
- [x] Slide editor: WYSIWYG canvas
- [x] Insert text boxes, images, shapes, tables, charts
- [x] Built-in themes and color palettes
- [x] Slide Master for custom templates
- [x] Animations (fade, fly-in, zoom) per object
- [x] Slide transitions (cut, fade, slide)
- [x] Speaker notes per slide
- [x] Presenter view: current slide, notes, timer, next slide preview
- [x] Export: PPTX, PDF, PNG/JPEG per slide
- [x] Import: PPTX (convert to native format)
- [x] File size limit: 100 MB (converted from PPTX)

### 3.4 Editor Infrastructure
- [ ] Auto-save (no manual save required)
- [ ] In-editor revision history (view, restore, name versions)
- [ ] Templates gallery (Docs: resume, letter; Sheets: budget, invoice; Slides: pitch deck, etc.)
- [ ] Offline editing via service worker / browser cache (Chrome/Edge)

### 3.5 Photos (Media Library)
- [ ] Dedicated Photos section: separate from Drive file browser, infinite-scroll grid by capture date
- [ ] Auto backup: mobile apps upload camera roll automatically (Wi-Fi only or cellular, user-configurable)
- [ ] Supported formats: JPEG, PNG, GIF, WebP, HEIC, RAW (common camera formats), MP4, MOV, AVI, WebM
- [ ] Albums: create manual albums; auto-albums for trips (location + date), events, and favorites
- [ ] Favorites: star photos/videos to collect in a "Favorites" album
- [ ] Archive: move items out of main stream without deleting
- [ ] Trash: 60-day retention before permanent deletion
- [ ] Shared albums: collaborative albums where multiple users can add photos and leave comments/likes
- [ ] Shared library: grant a trusted contact access to your full library (or a subset by person/date)
- [ ] Link sharing: share individual photos or albums via link; control whether recipients can save
- [ ] Export: download original-quality library via bulk export (ZIP)

---

## Phase 4 — Real-Time Collaboration

**Goal:** Multiple users can work in the same file simultaneously, with visibility and commenting.

### 4.1 Real-Time Co-Editing
- [ ] Operational transform or CRDT-based conflict-free editing engine
- [ ] Live cursors: show each collaborator's cursor position with name label
- [ ] Merge concurrent edits without conflicts
- [ ] Presence indicators (avatars of active collaborators)
- [ ] In-editor chat sidebar (visible while a file is open by 2+ people)

### 4.2 Comments & Mentions
- [ ] Add comment threads on text (Docs), cells (Sheets), objects (Slides)
- [ ] Reply to comments, resolve threads
- [ ] @mention a collaborator in a comment (triggers email notification)
- [ ] Assign action items from comments
- [ ] Comment history (chronological view of all comment threads)

### 4.3 Suggestion Mode (Docs)
- [ ] Toggle "Suggesting" mode — edits become tracked changes
- [ ] Accept or reject individual suggestions
- [ ] See who made each suggestion with timestamp
- [ ] Suggestion count indicator in toolbar

### 4.4 Notifications
- [ ] Email notification for: new comment, reply, @mention, suggestion accepted/rejected
- [ ] In-app notification center
- [ ] Push notifications (web) for comment/share activity

### 4.5 Activity Dashboard
- [ ] Per-file activity log: view edits, comments, shares chronologically
- [ ] Show who viewed the file and when (premium tier)
- [ ] File activity sidebar accessible from Drive and within editors

---

## Phase 5 — Desktop & Mobile Clients

**Goal:** Users have native access to Neutrino on desktop OS and mobile devices.

### 5.1 Mobile App — Drive (Android & iOS)
- [ ] Browse files and folders (list and grid view)
- [ ] Upload files from device camera roll or file system
- [ ] Download files for offline access
- [ ] Mark files "Available offline" (toggle per file or folder)
- [ ] Offline filter: view all offline-ready files
- [ ] Document scanning: use camera to scan paper docs into searchable PDF (OCR)
- [ ] File preview: images, video, PDF on mobile
- [ ] Share files via link or email from mobile
- [ ] Set permissions from mobile share dialog
- [ ] Open file in external apps ("Open with")
- [ ] Push notifications: comments, mentions, new shares, access requests
- [ ] Search Drive with voice input
- [ ] Biometric login

### 5.2 Mobile Editors (Android & iOS)
- [ ] Docs app: format text, insert images, comments, suggest mode
- [ ] Sheets app: edit cells, formulas, sort/filter, chart view
- [ ] Slides app: edit text, reorder slides, add images, present mode
- [ ] Offline editing in all three apps; sync on reconnect

### 5.3 Desktop Sync Client (Windows & macOS)
- [ ] Native installer (Windows .exe, macOS .dmg)
- [ ] Mount Drive as a virtual drive in Finder/Explorer
- [ ] **Stream mode:** files are cloud-only, downloaded on demand; offline-mark specific files
- [ ] **Mirror mode:** full local copy of all My Drive files, always offline-ready
- [ ] Switch between stream and mirror mode in settings
- [ ] Bidirectional sync: local edits upload, remote edits download
- [ ] Pause/resume sync
- [ ] Backup local folders to Drive (Desktop, Documents, Pictures, external drives)
- [ ] Multiple account support (up to 4 accounts simultaneously)
- [ ] Microsoft Office integration: DOCX/XLSX/PPTX files open in Office; changes sync to Drive
- [ ] Right-click context menu: share, sync status, open in browser
- [ ] Sync status indicators (cloud icon = online-only, green check = offline-ready)
- [ ] System tray icon with sync status and quick actions

---

## Phase 6 — AI and Advanced Features

**Goal:** Users can find files instantly and get intelligent assistance inside their documents and Drive. Adds advanced features to apps.

### 6.1 Search Infrastructure
- [ ] Full-text indexing: index content of Docs, Sheets, Slides
- [ ] OCR indexing: extract and index text from images and PDF files
- [ ] Search by file name, content, file type, owner, date range, location, shared status
- [ ] Natural language query support (e.g. "budget spreadsheet from last month")
- [ ] Fuzzy/typo-tolerant search
- [ ] Search result ranking by relevance and recency

### 6.2 Quick Access / Priority
- [ ] ML model to predict files a user is likely to need based on access patterns
- [ ] "Quick Access" section on Drive home showing top predicted files
- [ ] Suggested collaborators when sharing
- [ ] Suggested actions on file cards (e.g. "reply to comment", "view changes")

### 6.3 AI Writing Assistance (Docs)
- [ ] Smart Compose: autocomplete sentences and phrases while typing
- [ ] Grammar and style suggestions inline
- [ ] Voice typing: dictate text via microphone
- [ ] Translate entire document to another language
- [ ] Explore panel: suggest relevant web content and Drive files; auto-cite sources
- [ ] "Help me write" prompt: generate a draft from a user description

### 6.4 AI Data Assistance (Sheets)
- [ ] Smart Fill: detect column patterns and auto-fill based on examples
- [ ] Explore sidebar: natural language questions ("What is the sum of column B?") with chart/answer output
- [ ] Auto-generate pivot table from a text prompt
- [ ] Anomaly detection and data insights surfaced automatically

### 6.5 AI Presentation Assistance (Slides)
- [ ] Smart Compose: suggest completions for slide text
- [ ] Image search from within editor (search web or Drive)
- [ ] "Help me design": generate slide layout from a prompt
- [ ] Auto-format: balance text and images on a slide

### 6.6 Drive-Level AI
- [ ] Document summaries on hover/preview (AI-generated)
- [ ] "Catch me up" on Drive home: summarize recent changes across all files
- [ ] AI-powered content classification: automatically label sensitive files
- [ ] Answer questions about Drive content without opening files

### 6.7 Google Photo Advanced Feautures
- [ ] Photo map: browse media by GPS location on an interactive map
- [ ] Non-destructive editing: brightness, contrast, saturation, warmth, highlights, shadows, crop, rotate, filters; original always preserved and restorable
- [ ] Video editing: trim, stabilize, mute audio; export edited clip
- [ ] Semantic search: search by object, place, date range, or activity without manual tagging
- [ ] Memories: "On this day" highlights, year-in-review, and themed collages surfaced automatically
- [ ] Locked folder: PIN/biometric-protected private space excluded from backups and search
- [ ] Location privacy: strip GPS metadata before sharing; disable location in shared links
- [ ] Free up space: identify backed-up photos and offer to delete local device copies

### 6.8 Google Docs Advanced Features
- [ ] Template support: Can create, edit, use, and select a default template

---

## Phase 7 — Security, Administration & Compliance

**Goal:** Enterprise and team administrators can govern data, enforce policies, and maintain compliance.

### 7.1 Core Security
- [ ] Encryption at rest (AES-256) and in transit (TLS 1.3)
- [ ] Two-factor authentication (TOTP, hardware keys)
- [ ] SAML SSO integration
- [ ] Session management: revoke active sessions per device

### 7.2 Admin Console
- [ ] Org-level admin panel: manage users, groups, and storage allocation
- [ ] Enforce sharing policies (block external shares, restrict to domain)
- [ ] Context-Aware Access: device-signal-based access policies (managed device required, etc.)
- [ ] Enforce 2FA org-wide
- [ ] Set default IRM restrictions for all files

### 7.3 Shared Drives (Team Drives)
- [ ] Create team-owned drives not tied to any individual
- [ ] Roles: Manager, Content Manager, Contributor, Commenter, Viewer
- [ ] Add/remove members per shared drive
- [ ] File ownership stays with the drive if a member leaves
- [ ] Shared drive storage analytics (used space, contributor breakdown)

### 7.4 Data Loss Prevention (DLP)
- [ ] Define DLP rules: detect sensitive content (SSN, credit card, PII patterns)
- [ ] Auto-restrict sharing of files matching DLP rules
- [ ] Notify admins when DLP violation occurs
- [ ] AI-powered content classification to auto-label sensitive files

### 7.5 Audit & Compliance
- [ ] Drive audit log: record all file actions (view, edit, download, share, delete)
- [ ] Admin access to audit logs via console and export (CSV, API)
- [ ] Legal hold / data retention policies (archive files beyond normal deletion)
- [ ] eDiscovery: search archived data across all users
- [ ] Compliance certifications: target SOC 2 Type II, ISO 27001, GDPR, HIPAA

### 7.6 Advanced Security (Enterprise)
- [ ] Customer-managed encryption keys (CMEK)
- [ ] Ransomware detection: identify mass-encryption events and offer snapshot restore
- [ ] Security Command Center integration for threat detection
- [ ] Audit export to SIEM systems

---

## Phase 8 — Integrations & Ecosystem

**Goal:** Neutrino connects to tools users already use and offers an extensible platform for third-party developers.

### 8.1 Google Workspace / Office Suite Integration
- [ ] Embed Sheets charts into Docs and Slides (live-linked, auto-updating)
- [ ] Link Drive files in calendar events
- [ ] Attach Drive files in email compose (no download)
- [ ] Save email attachments directly to Drive (one-click)
- [ ] Meeting recordings auto-save to Drive

### 8.2 Third-Party App Connectors
- [ ] Native Slack integration: share Drive files, receive Drive notifications in Slack
- [ ] Native Zoom integration: present Slides directly into Zoom
- [ ] DocuSign integration: e-sign documents in Drive
- [ ] Salesforce connector: attach Drive files to Salesforce records
- [ ] Import from Dropbox and Box

### 8.3 Developer Platform (Drive API)
- [ ] REST API: CRUD operations on files, folders, permissions, comments
- [ ] Webhooks / push notifications for file change events
- [ ] OAuth 2.0 for third-party app access
- [ ] API rate limits and quota management
- [ ] SDK libraries (Python, JavaScript/TypeScript, Go, Java)
- [ ] API documentation and developer portal

### 8.4 Add-on Marketplace
- [ ] In-editor add-on framework (Docs, Sheets, Slides)
- [ ] Marketplace listing: discover and install add-ons
- [ ] Add-on sandboxing and permission scopes
- [ ] Featured add-ons: diagram tools, mail merge, CRM connectors

---

## Phase Summary

| Phase | Focus | Key Milestone |
|---|---|---|
| 1 | Core Storage & File Management | Users can store, browse, and preview files |
| 2 | Sharing & Permissions | Users can share files with role-based access |
| 3 | Native Editors | Create and edit Docs, Sheets, Slides natively |
| 4 | Real-Time Collaboration | Multiple users edit simultaneously with comments |
| 5 | Desktop & Mobile Clients | Native apps on Windows, macOS, Android, iOS |
| 6 | Search & AI Features | Instant search and AI writing/data assistance |
| 7 | Security & Compliance | Enterprise admin, DLP, audit logs, compliance certs |
| 8 | Integrations & Ecosystem | Third-party connectors and developer API |
