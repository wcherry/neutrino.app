# Google Drive – PRD-Style Feature Map  

A comprehensive breakdown of Google Drive’s features (core storage, file management, sharing, collaboration, security, etc.) across Web, Desktop, and Mobile clients. This includes Google Docs, Sheets, and Slides features and all relevant limits. Each item is sourced from Google documentation and Workspace updates.  

## Storage & Data Limits  
- **Default Storage Quota:** 15 GB per user on free accounts (shared across Drive, Gmail, Photos)【47†L599-L602】. (Workspace paid plans scale from 30 GB up to 5 TB per user; Enterprise plans can request even more【47†L599-L602】.)  
- **File Size Limits:** Supports files up to **5 TB** each【11†L73-L76】【15†L981-L986】. (Daily upload/copy limit is 750 GB per user【15†L981-L986】; files over 750 GB cannot be uploaded until 24h passes.)  
- **Drive File Type Limits:** No strict number-of-files limit; practical limits are high (shared drives hold ~400k+ items). Each Google Docs file: ~**1.02 million characters**【11†L41-L44】. Google Sheets: up to **10 million cells** (in any combination of sheets)【11†L46-L54】. Google Slides (converted from PPTX) up to **100 MB**【11†L57-L60】.  
- **Supported Formats:** Almost any file can be stored. Native editors (Docs, Sheets, Slides) auto-convert or open Office (DOCX/XLSX/PPTX) and PDF files. Over **100 file types** can be previewed: PDF, Office, images (JPEG/PNG/GIF/TIFF/RAW), audio (MP3, WAV), video (MP4, MOV), text (TXT, RTF), ZIP archives, source code, CAD, Photoshop (PSD) and more【47†L648-L652】【11†L79-L88】. (Non-Google files open in default apps or previews.)  
- **Version History:** Automatic revision history for all native files: each edit is saved (you can name and restore versions)【35†L149-L158】. For non-native files, Drive retains previous versions (up to 100 versions for large files). Users can view file activity and restore deleted items from Trash (files remain 30 days by default).  
- **File & Folder Organization:** Unlimited nested folders. Users can create folders and subfolders, drag-and-drop files, and “Star” items for quick access. Folders and shortcuts can be color-coded. Shortcuts let one file appear in multiple folders without duplication. Bulk actions: multi-select for move, download, delete, or change sharing settings. Each file/folder shows metadata (owner, last modified, size, location, version count) in an info panel.  
- **Offline Availability:** Drive supports offline access. On the Web, users enable Offline mode (Chrome/Edge extension) to open/create/edit recent Docs/Sheets/Slides without Internet【25†L49-L57】. Files marked “Available offline” in Drive or the Docs/Sheets/Slides apps sync to local storage. On mobile apps, any recently accessed or explicitly chosen file can be kept offline (accessible via an “Offline” filter)【26†L43-L51】. Drive for Desktop’s **Mirror** mode keeps a local copy of My Drive files (always available offline)【31†L31-L39】.  
- **Sync & Backup (Desktop):** Google Drive for Desktop app synchronizes Drive with your PC. Two modes: **Stream** (files stay in cloud until opened, minimizing disk use) and **Mirror** (a full local copy)【31†L31-L39】. Users can choose which folders from Drive to stream/mirror. The app also **backs up** specified local folders (Desktop, Documents, Photos, external drives) to Drive. Edits in one place sync everywhere. Sync can be paused/resumed【30†L188-L196】. The app appears as a mounted drive in Explorer/Finder, offering direct access to “My Drive”, “Shared drives”, and “Computers” (for backup sets).  
- **File Preview & Editing:** Drive has built-in viewers for PDFs, images, videos, audio, Microsoft Office files, text, and ZIPs. Users can preview without downloading. Docs/Sheets/Slides open in browser for editing. Office files can also be edited in-place via Office Add-in (real-time co-authoring in Word/Excel【1†L61-L67】). PDFs can be annotated: highlight text and add comments directly in Drive【47†L656-L660】.  

## Search & AI Assistance  
- **Full-Text Search:** Global search across Drive (file names, contents of Docs/Sheets/Slides, and OCR’d text in images/PDFs). Filters for type (document, spreadsheet, image, etc.), owner, date, location, and shared-with. Natural-language queries (e.g. “presentations May 2025”) retrieve relevant files. Search ranking uses relevance signals.  
- **Priority / Quick Access:** The web and mobile homepage show recommended files (“Priority”) based on recent/frequent use and collaborators【17†L84-L92】【47†L662-L668】. Priority also suggests quick actions (reply to comments, view attachments, grant access) using AI/ML on your activity history【17†L84-L92】【17†L115-L122】.  
- **Gemini Summaries & AI Features:** New in 2024–2026, Drive integrates Google’s Gemini AI assistant. On file lists, Drive can *summarize long documents*, highlight important points, and answer questions about your files【47†L587-L592】. In Search, Gemini can surface key facts from your Drive content without opening files. This helps “catch up” on changes and extract insights without manual reading.  
- **AI-Powered Search:** Drive’s search itself is AI-augmented, quickly surfacing the most relevant files based on activity signals【47†L662-L668】. It learns from edits, comments, and accesses to rank search results intelligently.  

## Sharing & Collaboration  
- **Link Sharing & Permissions:** Users can share files/folders by link or explicit invitation. Roles: **Owner** (full control), **Editor**, **Commenter**, **Viewer**. Editors and commenters can be further restricted by Information Rights Management (IRM) so they *cannot* download, print, or copy content【23†L89-L98】. Shares can be set to anyone (public), anyone in a Google Workspace domain, or specific people. Shared items show in a special “Shared with me” section.  
- **Expiration & Access Management:** On Drive (and Docs/Sheets/Slides), sharers can set link expiration dates and disable options like copy/download/print【47†L688-L692】. Admins can enforce domain-only sharing or block external shares. Shared items show active viewers and recent sharing activity. Users can revoke access or transfer ownership at any time.  
- **Folder-Level Sharing:** Sharing a folder automatically shares its contents with the same permissions. Items inherit permissions from parent folders.  
- **Real-Time Co-Editing:** Multiple users can simultaneously edit a file (Docs/Sheets/Slides). Each user’s cursor is visible (showing name), and edits merge in real time. Conflicts are auto-resolved. Collaboration features include built-in chat (via Hangouts/Chat sidebar) and presence indicators.  
- **Comments & Suggestions:** In Docs, Sheets, Slides, users can add comment threads on text, cells, or objects and assign them to collaborators (using **@-mentions** to notify specific people)【35†L101-L104】【37†L85-L89】. Commenters can reply, resolve, or turn comments into action items/tasks. Docs has a “Suggestion” mode (tracked changes) where edits can be reviewed and accepted.  
- **Request Access & Notifications:** If someone tries to access a restricted file, the owner is notified and can grant access. Drive sends email/push notifications for shared files, comments, and access requests. Activity dashboard (premium) gives a unified view of who viewed or edited a file recently【47†L670-L678】.  
- **Shared Drives (Team Drives):** For Workspace accounts, **Shared drives** allow teams to own files collectively (content lives in the drive, not under an individual)【19†L328-L333】【47†L696-L704】. Members are granted roles (Manager, Content Manager, Contributor, Commenter, Viewer) – note: there is *no* single “owner” role in a Shared Drive【19†L378-L386】. Shared drive settings let managers add/remove members, set access levels, and restrict sharing. Files in a Shared Drive stay with the team even if the original uploader leaves.  
- **Comments on Non-Native Files:** PDF files stored in Drive can be annotated with comments【47†L656-L660】. Users highlight text or images in PDFs and add comments just like in Docs. (Drive also preserves comments if a PDF is converted to a Docs file.)  

## Core Drive Features (Web UI)  
- **Upload & Create:** “New” menu to upload files/folders from PC or create new Docs, Sheets, Slides, Forms, Drawings, Tables (Teams feature), and Google Sites. Drag-drop also triggers upload. Google Forms and Sites autosave in Drive.  
- **View Modes:** Toggle between List view and Grid (large thumbnails). Sort by Name/Date/Owner/Size. Filter by file type or starred/shared.  
- **File Preview:** Click a file to preview. Previews support video (with playback), images, Office docs, PDFs, text, and code syntax highlighting. Users can print or download from the preview.  
- **File Operations:** Context menu (right-click or three-dot): Move to Folder, Add Shortcut, Rename, Star, Color-label, Organize (drag), Download, Make a copy (for Google files, saving a new copy), Open with (drive-integrated apps or third-party services).  
- **Manage Versions:** For non-Google files, you can upload a new version (keeping history) or revert to an older version.  
- **File Shortcuts:** Instead of duplicating files, users can “Add shortcut to Drive” (link a file or folder to multiple locations). Shortcuts sync with the original.  
- **Offline Files (Web):** In Drive settings, enabling Offline makes recent Google files available offline. A special “Offline” filter shows which files are ready for offline use【25†L94-L102】.  
- **Activity & Details Sidebar:** Clicking a file shows a sidebar with activity (recent edits, comments), version history, and view history (who viewed the file and when).  

## Google Docs (Editor) – Key Features  
- **Rich Editing:** Text formatting (styles, fonts, colors), lists, tables, images, drawings, shapes, headers/footers, footnotes, equations, pagination. Document outline (based on headings) for navigation.  
- **Real-Time Collaboration:** Multi-user editing with live cursors. See collaborators’ avatars. Everyone edits the same document live. Changes auto-save (no manual save).  
- **Comments & Suggestions:** Insert comments on text. “Suggesting” mode lets collaborators propose edits (tracked changes) that can be accepted/rejected. @-mention users (updates email them).  
- **Revision History:** View all past versions, restored points, and named versions. Track when each user made edits and revert if needed.  
- **Formatting Tools:** Styles (Normal, Heading 1–6), font formatting, highlighting. Automatic Table of Contents generation. Page setup (margins, orientation, page size). Columns.  
- **Add-ons & Extensions:** Support for Google Workspace Marketplace add-ons (e.g. mail merge, e-signature, diagrams). Many apps integrate via the “Add-ons” menu.  
- **Voice Typing & Smart Compose:** Dictate text via microphone. Grammar/smart compose suggestions help write faster (e.g. phrase completion, grammar fixes)【35†L110-L114】.  
- **Translation & Research:** Built-in translator for entire doc. Explore panel suggests relevant content from the web/Drive and auto-cites. Dictionary, word count, and “Research” tools.  
- **Templates:** A gallery (resume, business letter, newsletter, etc.) to start. Custom doc templates via Drive (premium).  
- **Offline Editing:** Docs can be edited offline (via Chrome/Edge with extension)【25†L49-L57】. Offline changes sync when reconnected.  
- **Export Formats:** Download as DOCX, PDF, ODT, RTF, TXT, HTML, EPUB. Can upload DOCX/RTF/ODT to convert to Google Docs (50 MB limit for conversion)【11†L41-L44】.  

## Google Sheets – Key Features  
- **Spreadsheet Functionality:** Standard grid interface with rows/columns. Supports cell ranges, merged cells, hiding columns/rows. Freeze panes (rows/columns). Sorting and filtering (Filter views and slicers for collaborative filters).  
- **Formulas & Functions:** 400+ built-in functions (math, stats, text, date, financial, logical, lookup, engineering, etc.). Cell references (A1, R1C1). Auto-complete function names. Formula autoupdate on changes.  
- **Smart Fill & AI:** Smart Fill guesses patterns (e.g. auto-fill column based on examples)【37†L93-L100】. “Explore” (AI insights) allows natural-language questions (e.g. “What is total sales?”) and auto-generates charts/answers.  
- **Pivot Tables & Analysis:** Create pivot tables from data. (Connected Sheets supports pivoting up to 200k rows【11†L52-L54】 for BigQuery data).  
- **Charts & Graphs:** Insert charts (line, bar, column, pie, area, scatter, maps, etc.). Customize titles, axes, legends, series colors. Charts update as data changes.  
- **Conditional Formatting:** Set color rules based on cell values (gradients, color scales). Data validation rules (dropdown lists, numeric restrictions).  
- **Collaboration:** Multi-user editing (live cursors). Cell-based comments. Version history with individual cell edit tracking. Protected ranges (lock cells or ranges from editing by certain users).  
- **Import/Export:** Import CSV, TSV, Excel (XLSX) without conversion (open in Sheets natively)【37†L110-L114】. Functions to import data from other sheets. Google Forms can feed responses into a sheet.  
- **Add-ons & Scripts:** Support custom macros (record-and-playback). Google Apps Script for automation (custom functions, menus, triggers). Marketplace add-ons (analysis, CRM, mail merges).  
- **Connected Sheets:** (Enterprise) Link BigQuery datasets live in Sheets (billions of rows, auto-aggregate)【49†L138-L142】. Real-time data connectors (Analytics, Salesforce).  
- **Offline Editing:** Sheets can be edited offline (Chrome/Edge)【25†L49-L57】 and on mobile apps【26†L43-L51】.  
- **Templates:** Many pre-built templates (invoicing, budgets, schedules) help users start common tasks.  

## Google Slides – Key Features  
- **Slide Creation:** WYSIWYG slide editor. Insert text boxes, images, shapes, tables, charts (linked from Sheets), diagrams. Draw lines, arrows.  
- **Themes & Templates:** Choose from built-in themes and color palettes. Master Slides feature for company templates (custom fonts, logos, slide layouts). Templates gallery (pitch decks, education, etc.).  
- **Animations & Transitions:** Build animations (fade, fly-in, spin) for slide objects; set transitions (cut, slide, fade) between slides.  Animation pane to reorder sequences.  
- **Collaboration:** Real-time co-editing (live cursors). Comments on slides or individual elements. Revision history per slide. Chat sidebar during multi-user edits.  
- **Presenter Tools:** Presenter view (current slide + notes + next slide preview + timer) during presentation. Speaker notes panel. Q&A feature to gather audience questions. Live captions (speech-to-text) appear during presentation.  
- **Embed & Export:** Embed video (YouTube) and audio. Link charts from Sheets (updateable). Export entire deck or individual slides as PPTX, PDF, PNG/JPEG.  
- **Compatibility:** Import PowerPoint (PPTX) files; slides become fully editable (with animations). Export as PPTX. Can present to Meet/Zoom directly from Slides.  
- **Smart Compose:** Suggested slide content phrasing and autocorrect for text in slides【38†L129-L134】.  
- **Offline:** Slides can be edited offline (web offline mode)【25†L49-L57】 and on mobile (choose slides to keep offline).  
- **Templates:** Diverse slide templates for business, education, etc., accessible from creation menu.  

## Google Drive (Desktop Client)  
- **Drive for Desktop App:** Native app for Windows/macOS that syncs Drive with a local folder or virtual drive. Installed via download.  
- **Streaming Mode:** Only selected files download on demand; others appear as placeholders to save disk space【31†L31-L39】. Access when online, or mark specific files for offline.  
- **Mirroring Mode:** Keeps a full local copy of My Drive files (and any backup folders), always available offline【31†L31-L39】. Automatically syncs all changes.  
- **Multiple Accounts:** Supports multiple Google accounts in one app (up to 4). Easy switching to view each account’s Drive.  
- **Office Integration:** Microsoft Office files in Drive can be edited with Office apps; changes auto-sync to Drive【1†L61-L67】. Outlook can save email attachments directly to Drive folder.  
- **Sync Controls:** Pause/resume sync. Select which local folders to back up to Drive (desktop, docs, pictures, external drives). View sync status for files. The “Sync status” tile in the app shows recent syncs.  
- **File Explorer/Finder Integration:** Files appear under a “Google Drive” drive or folder in the OS. Native file search (Windows Search, macOS Spotlight) can find files, though Drive’s own search (Ctrl+Alt+G / ⌘+⌥+G) searches in Drive storage【30†L180-L187】. Right-click to share or open with apps.  
- **Online-only Files:** In stream mode, files not downloaded are marked with a cloud icon; double-clicking downloads them. Offline-marked files show a green check.  
- **Shared Drives Support:** Both stream and mirror modes can show Shared drives (stream-only if chosen).  
- **Versioning:** Local version history is minimal (Focus is cloud versioning).  
- **Photos/Backup:** Option to also back up Photos to Google Photos.  

## Mobile (Android/iOS) Drive App  
- **Browse & Manage:** View all Drive files and folders. List or grid view. Sort/filter (e.g. file type, owner).  
- **Upload/Download:** Upload from camera or gallery. Save offline (toggle “Available offline”). Download files for use in other apps.  
- **Document Scanning:** Use camera to scan documents/receipts into PDF. OCR (searchable text) is applied automatically【47†L615-L619】.  
- **Offline Access:** Recent and manually selected files can be kept offline for later access【26†L43-L51】. Offline files sync changes when online.  
- **File Sharing:** Share via email or link. Set viewer/editor permissions. Share to contacts or other apps.  
- **File Previews:** Preview images, videos, PDFs on mobile (slideshows for images, video player).  
- **Open in Other Apps:** “Open with” to use other mobile apps (e.g. MS Word, Adobe Reader). Edits can sync back if saved to Drive.  
- **Notifications:** Push alerts for shared file activity (comments, mentions, requests).  
- **Camera Uploads (Photos):** Option to auto-sync camera roll images to Drive (alternate to Google Photos).  
- **Search:** Drive mobile search; voice search supported.  
- **Integrated Tools:** Access Google Docs/Sheets/Slides to edit documents. “Add comment” in PDF with highlighting.  

## Integrations & Extensions  
- **Office Compatibility:** Seamlessly open/edit Office docs: Docs edits Word (.DOCX) online (no conversion needed)【35†L129-L133】; Sheets edits Excel (.XLSX)【37†L110-L114】; Slides edits PowerPoint (.PPTX)【38†L112-L117】.  
- **Gmail Integration:** Save email attachments to Drive (one-click)【47†L607-L610】. Open Drive files directly from Gmail (Docs Editor). Attach Drive links to emails without downloading.  
- **Calendar/Meet Integration:** Attach Drive files to Calendar events. Present Slides directly into Google Meet. Meeting recordings (Meet) can auto-save to Drive.  
- **Third-Party Apps:** Drive has built-in connectors: import from Dropbox, Box, Salesforce, etc. Also, third-party apps can integrate via “Open with” (e.g. DocuSign for e-signatures). New “Google Drive app for Slack/Zoom” lets sharing into those platforms. (Premium plan: Native Slack, Zoom, Salesforce integrations【47†L720-L729】.)  
- **Add-ons & Plugins:** From Google Workspace Marketplace, install add-ons that extend Drive and Editors (e.g. docu sign, diagram tools). Drive API allows custom apps to read/write Drive content.  
- **Google Workspace App Suite:** Drive links seamlessly with all Workspace apps: link Docs in Chat, embed Sheets charts in Docs/Slides【35†L119-L124】【37†L101-L106】, use Forms with Sheets, keep notes in Keep attached to Drive.  

## Security & Administration  
- **Encryption:** All files are encrypted in transit and at rest by default【35†L171-L174】【37†L153-L156】. (Workspace supports Customer-Managed Encryption Keys for extra control.)  
- **Data Loss Prevention:** Admins can set DLP rules on Drive content (detect sensitive info like SSNs, block sharing). Enhanced IRM via Workspace Admin can automatically prevent downloads/print for sensitive files.  
- **Access Controls:** Admin console can restrict sharing to within organization. Context-Aware Access (device signals) can limit access. Admins can enforce 2-step verification on accounts.  
- **IRM/DLP:** Drive file owners and admins can disable download/copy/print on a per-file basis【23†L89-L98】. In 2025, Drive extended these restrictions so even Editors can be blocked from copying【23†L89-L98】.  
- **Audit & Reports:** Audit logs track file actions (view, edit, share). Admins can see which files have been shared externally or edited. Drive admins get usage reports (storage growth, shares, etc.).  
- **Ransomware Detection:** Advanced security features can detect encryption attacks on Drive and offer file restoration options (rolled out in 2023).  
- **Compliance:** Google Drive complies with standards (ISO 27001, SOC, FedRAMP, HIPAA, FERPA). Features like Vault allow setting retention policies and legal holds on Drive data.  
- **Authentication:** Supports SAML SSO. Admins manage device policies (block jailbroken devices, etc.). Team Drives (Shared drives) have member role controls (Content Manager, Organizer, etc.)【19†L378-L386】.  
- **Customer-managed Keys:** Enterprise Plus can use customer-supplied encryption keys (CSE).  
- **Security Notices:** Drive integrates with Google’s Security Command Center for threat detection in Drive files.  

## Limits & Quotas  
- **Storage per User:** 15 GB free; upgrade to 100 GB, 200 GB, 2 TB (One plans); up to 5 TB or unlimited (Workspace)【47†L599-L602】. Additional storage is paid.  
- **File Upload Size:** Max 5 TB per file【11†L73-L76】. Only first file >5 TB will complete uploading; rest fail.  
- **Daily Upload Cap:** 750 GB of uploads (My Drive + Shared drives) per user per day【15†L981-L986】. After hitting cap, user cannot upload/copy new files for 24 hours.  
- **Document Size:** Google Docs ~1.02M characters【11†L41-L44】 (≥200 pages). Google Sheets 10M cells【11†L46-L54】. Google Slides 100 MB converted size【11†L57-L60】. (Docs/Sheets/Slides large cells/content may degrade performance.)  
- **File Count:** No fixed limit on total files. Performance note: having millions of files in one folder may cause UI slowdown. Shared drive limit ~400k items. Maximum 20 levels of folder nesting recommended.  
- **API Quotas:** Drive API: 12,000 queries/min (per project/user)【15†L976-L984】. Per-user daily limit as above.  
- **Versioning:** Non-Google files: up to 100 file versions per item (oldest pruned beyond that).  
- **Business Accounts:** Shared drives (Team drives) require Business/Education plans. Certain features (Activity dashboard, Drive labels) need Enterprise plans.  
- **Other Limits:** Forms: up to 2M responses; Sites: up to 200 000 pages; Vids (beta) limits pending.    

## Platform-Specific Highlights  
- **Web (drive.google.com):** Full-featured interface. Supports add-ons, templates, comprehensive search, and advanced sharing. Can work offline (with Chrome/Edge)【25†L49-L57】. The web UI shows Google Docs/Sheets/Slides icons next to native files, and integrates AI tools.  
- **Desktop (Drive for Desktop):** Synchronizes Drive with local OS. Enables streaming/mirroring, right-click sharing, and background backups. Good for large-volume sync and offline file access. Integrates with Windows/Mac file system.  
- **Mobile (Android/iOS):** Drive, Docs, Sheets, Slides have dedicated apps. Mobile Drive app lets you upload photos (including scanning to PDF)【47†L615-L619】, share on the go, and view offline content. Mobile editors offer core editing (format text, insert images, resize objects) though some desktop features (add-ons, complex formatting) are limited. Push notifications keep users aware of file activity.

## Notable Add-on Features  
- **Gmail Integration:** One-click “Save to Drive” for attachments【47†L607-L610】; preview Drive files in Gmail compose.  
- **PDF Comments:** Highlight text in PDFs and add comments in Drive【47†L656-L660】 (transfers to Docs if converted).  
- **Templates:** Central template galleries for Docs/Sheets/Slides (built-in and custom) to standardize documents, spreadsheets, and presentations.  
- **Offline Folders (Mobile):** Entire folders can be made available offline on mobile.  
- **Shortcuts & Linking:** “Add shortcut” allows linking files into multiple folders. Files can link to Google Calendar, Chat, Gmail, etc.  
- **Shared Drives Analytics:** Shared drives show storage used and who’s contributed files. Team Drives can be locked for add-only by some members.  

## Summary  
Together, these features encompass Google Drive’s ecosystem for storage, file management, real-time collaboration, and integration. Google’s official documentation and blog posts confirm these capabilities【47†L599-L602】【35†L149-L158】【23†L89-L98】【25†L49-L57】【1†L54-L61】. This map outlines the granular feature set one would find in a Drive competitor’s PRD. 

