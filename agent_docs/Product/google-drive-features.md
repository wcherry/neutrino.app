# Google Drive & Editors – Feature Breakdown

Below is a **comprehensive feature map** for Google Drive and its Editors (Docs, Sheets, Slides), organized by component and platform. Citations to official Google documentation and updates are provided throughout.

---

## Core Google Drive (Cloud Storage)

- **Cloud Storage & Quotas.** Drive provides scalable cloud storage with free and paid tiers. Free accounts start at **15 GB** (shared across Drive, Gmail, Photos), while Workspace/Google One plans scale up to **5 TB per user** or more【13†L599-L602】. Individual files (non-Docs formats) can be up to **5 TB** in size【11†L73-L76】【15†L981-L986】. (The Drive API imposes a **750 GB/day** upload limit per user【15†L981-L986】.) Native Google Docs format has a ~**1.02 million character** limit per document【11†L41-L44】. Native Google Sheets are limited to **10 million cells** (≈18,278 columns)【11†L46-L54】. Google Slides converted from PowerPoint can be up to **100 MB**【11†L57-L60】.  

- **File Types & Previews.** You can store *any* file type in Drive; common types (Docs, Sheets, Slides, images, audio, video, archives, code files, etc.) can be previewed in-browser【11†L79-L88】. Office (DOCX/XLSX/PPTX) and PDF files open in Drive’s viewer or editors. Drive displays a scaled-down preview of most files (PDF, Office, images, video, ZIP, etc.) without downloading【11†L79-L88】.  

- **File Management.** Drive supports standard file/folder operations: nested folders, drag‐and‐drop organization, right-click context menus, star important items, and color‐coded folders. Each file shows metadata (name, owner, last modified, type, size, etc.), and a detailed version history (you can restore older versions or name versions). Deleted items go to Trash and auto-expunge after 30 days (or can be manually restored). Drive for Desktop (below) can backup local folders (Desktop, Documents, Photos) to the cloud.  

- **Versioning & History.** Every edit is auto-saved in Drive. For Google Docs/Sheets/Slides, a version history lets users see/restore prior states. For other files, Drive keeps previous revisions (e.g. older PDFs or images) for a limited time. Drive records file activities (edits, shares, comments) in an activity log for audit/history. Shared drives (see below) show file versioning similarly.  

- **Search & Discovery.** Drive’s search is **AI-augmented and full-text**: it indexes file names and content (OCR of images/PDFs, text inside Docs/Sheets/Slides). You can filter by type, owner, date, location, shared status, etc. Drive’s ML-powered *Quick Access/Priority* feature (on the web/mobile home screen) surfaces likely-needed files and suggests actions【17†L84-L92】【17†L115-L122】. In 2025, Google added Gemini-powered summaries on Drive home and in-search to give quick insights (e.g. “Catch me up” shows recent changes)【5†L249-L257】【5†L303-L307】.  

- **Sharing & Permissions.** Files can be shared with individuals, Google groups or anyone with a link. Permission roles include **Viewer, Commenter, Editor**, and Owner. Owners/editors can restrict downloading, printing or copying via Information Rights Management (IRM). By default, Docs/Sheets/Slides allow commenters/viewers only view access. Recently, Drive added IRM controls to **block download/print/copy even for users with edit access**【23†L89-L98】. Shared links can have expiration dates and domain limits. The Drive admin console can enforce enterprise DLP and context-aware access.  Workspace (paid) plans add controls like expiring access and block previews.  

- **Collaboration.** Drive is the nexus for real-time collaboration. Google Docs Editors (Docs/Sheets/Slides) allow **simultaneous editing** by multiple users. In-Editor features (comments, suggestions, action items, chat) are stored in the file. You can @-mention users in comments to notify them【35†L101-L104】【37†L85-L89】. Comment threads and suggestions are tracked and can be resolved or accepted. Edits sync instantly so all collaborators see changes live. Drive’s “Activity dashboard” (on Workspace Enterprise) shows who viewed or commented on files.  

- **File Sharing Summary (Workspace site).** Google highlights: *“Easily share files with customizable permissions (edit, comment, view). Control access further by preventing unwanted actions and setting expiration dates.”*【13†L689-L693】. You can share at folder-level and manage nested permissions. Files inside *Shared drives* (team drives) are owned by the team and have similar permission roles (writer, commenter, organizer)【19†L328-L333】【19†L378-L386】.  

- **Backup & Sync (Drive for Desktop).** The Drive for Desktop client (Windows/macOS) lets you *sync* or *stream* Drive on your PC: 
  - **Streaming mode:** Files live in the cloud and are downloaded on-demand (local storage only used for accessed files). Offline copies are automatically cached when opened【31†L31-L39】. 
  - **Mirroring mode:** My Drive files are fully copied to a local folder, always available offline; changes sync back to cloud【31†L31-L39】. 
  - You can switch modes in settings. Any edits to streamed or mirrored files on one device sync everywhere【31†L39-L42】. The desktop app appears as a mounted Drive in Finder/Explorer, showing “My Drive,” “Shared drives,” etc【30†L192-L199】. It supports multiple accounts and can back up chosen PC folders to Drive. It also integrates with Microsoft Office: Office files open in Office apps and changes sync to Drive【1†L61-L67】.  

- **Mobile (Drive app).** The Google Drive app (Android/iOS) lets you browse your files, upload photos/videos, share links, and make files available offline. Key mobile-only features include: **document scanning** (use camera to scan receipts/docs into PDF)【13†L615-L619】, uploading Gmail attachments directly to Drive, and push notifications for comments and shares. Offline mode (via a star or “make available offline”) lets you open documents/Sheets/Slides offline. The Drive app can preview and open 100+ file types on mobile, but full editing must switch to the Docs/Sheets/Slides apps.  

- **Account & App Integrations:** Drive integrates with Gmail (save attachments to Drive【13†L607-L610】), Google Photos, Calendar (Drive attachments in events), and 3rd-party add-ons (e.g. DocuSign, Lucidchart via Workspace Marketplace). It also supports creating Google Forms and Sites that store data in Drive.  

---

## Google Docs Editors (Docs, Sheets, Slides)

### Google Docs (Word Processor)
- **Real-Time Editing:** Simultaneous multi-user editing in rich-text documents. Any user’s cursor appears to collaborators, and text edits merge live. Changes auto-save constantly.  
- **Formatting & Content:** Supports text styles (bold, italics, headings), lists, columns, tables, images, drawings (via embedded Drawing), shapes, links, footnotes, citations, headers/footers, page numbers, and more. There is a document outline and table-of-contents feature.  
- **Collaboration Tools:** Comments threads, suggested edits (tracked changes), @-mentions to tag collaborators. Comments can be resolved or turned into action items. A **comment history** shows who said what when. Users can chat within the document if multiple people are in it at once. All suggestions can be accepted or rejected. *“Anyone’s working on the latest version”* – a single source of truth【35†L149-L158】. 
- **Assistive Writing Tools:** Built-in spell check, grammar check, Smart Compose (suggests completions)【35†L110-L118】, grammar suggestions, voice typing (speech-to-text), word count, and translation. Docs uses machine learning to suggest smarter writing. 
- **Intelligence & Add-ons:** “Explore” panel (AI-driven) can suggest relevant documents or images from Google, auto-generate content outlines, or summarize text. You can use the *Research* tool (look up and cite info). Supports add-ons/plugins (e.g. e-signature, diagramming, citation managers) for extended functionality【35†L140-L144】. 
- **Sharing & Security:** Docs files inherit Drive’s share settings. Within a Docs file, IRM prevents viewers/commenters from printing/downloading/copying【23†L89-L98】. Version history logs every save; older versions can be named or restored【35†L149-L158】. Comment email notifications go to collaborators.  
- **Offline:** You can mark Docs for offline use; in Chrome/Edge this is enabled in Drive settings【25†L49-L57】. Once offline, you can create/view/edit recent documents【25†L36-L44】. (On mobile, recent docs can be made offline in the app【26†L36-L44】.)  
- **Platform Parity:** The web version has the full feature set. Mobile apps (Android/iOS) support most editing features (formatting, comments, basic layouts) but have a simplified UI. A subset of features (e.g. add-ons, some advanced formatting) is only on web.  
- **Limits:** A Docs file (converted from text) can hold ~1.02 million characters【11†L41-L44】. (Web-based Google Docs has no fixed page or word limit beyond this.) When exporting, Docs can be saved as PDF, DOCX, ODT, TXT, HTML, EPUB, etc. It can import DOCX/RTF/ODT files (up to 50 MB if converted【11†L41-L44】).  

【35†L101-L104】【35†L110-L114】

### Google Sheets (Spreadsheet)
- **Grid & Data:** Classic spreadsheet grid with cells. Full support for formulas (SUM, VLOOKUP, QUERY, and hundreds more), functions, and custom functions via Apps Script. Includes data tools: sort, filter, find/replace, data validation.  Connected Sheets (for Enterprise) can pull BigQuery data and has no fixed row limit (billions of rows)【37†L136-L140】.  
- **Collaboration:** Multi-user editing with per-cell indication of collaborators. Comments and notes can be attached to cells; @-mentioning a user in a comment notifies them. All editors see changes in real-time. Version history lets you restore previous spreadsheet versions or see cell-by-cell edit history【37†L128-L132】.  
- **Insights & Intelligence:** Smart Fill predicts patterns and auto-completes columns from examples【37†L93-L97】. There is an *“Explore”* sidebar for analytics: ask questions in natural language (“What’s the sum of column B?”) and get answers/charts. Pivot tables and charts are built-in (basic pivot tables handle up to 200,000 rows in Connected Sheets【11†L52-L54】). Conditional formatting, goal seek, and solver are available.  
- **Charts & Visualization:** Create line, bar, pie, scatter charts, and more. Charts can be embedded and edited. Color-schemes and chart editors help customize visuals.  
- **Integration:** Sheets easily imports CSV/Excel data. It’s integrated with Google Forms (form responses can feed into a Sheet) and Google Finance (fetch stock data). One can insert charts from Sheets into Docs/Slides【37†L101-L106】, or embed Sheets into Sites.  
- **Automation:** Support for macros (record and replay), and Apps Script for complex automation (custom menus, functions, triggers). Users can add-ons for specialized tasks (e.g. Dataloop, QR code generation).  
- **Offline:** Like Docs, Sheets can be enabled offline in Drive settings【25†L49-L57】. Mobile app allows marking sheets offline【26†L36-L44】.  
- **Excel Compatibility:** You can open/edit Excel (XLSX) files directly without conversion【37†L110-L114】. Edits sync and collaborators can work on them in real-time (with some feature limitations for very advanced Excel-only functions).  
- **Limits:** Maximum of **10 million cells** per spreadsheet (sum of cells across all sheets)【11†L46-L54】. Any pivot table within Connected Sheets is limited to 200,000 rows【11†L52-L54】. When converting or importing, cells over 50,000 characters per cell are dropped. Sheets auto-saves and keeps version history【37†L128-L132】.  

【37†L85-L89】【37†L93-L97】

### Google Slides (Presentation)
- **Slide Creation:** WYSIWYG slide editor with text boxes, images, shapes, tables, charts (linked from Sheets), videos (YouTube), and diagrams. Choose from built-in **themes and layouts**; a Slide Master feature allows company-wide templates. Supports animations and slide transitions. You can add speaker notes and set automatic timings.  
- **Collaboration:** Real-time co-editing of slides with live cursors. Comments and suggestions can be attached to slides or objects. Action items can be assigned from comments. Version history records all changes【38†L123-L127】.  
- **Presentation Features:** Presenter view with current/past slides, speaker notes, and a timer. Live captions (speech-to-text) can display during presentations. Slides can present directly into Google Meet or as a web link. Files can be exported as PPTX, PDF, PNG/JPEG per slide, or text outline.  
- **Intelligence & Integration:** Slides has *Smart Compose* suggestions to finish sentences or list items【38†L129-L134】. You can search the web or Drive for images and content directly from the Slides editor【38†L102-L107】. As noted, you can embed charts from Sheets or Docs, and reply to Gmail comments【38†L102-L107】.  
- **PowerPoint Compatibility:** Import and export PPTX files. You can open a PPTX and continue editing in Slides (including collaboration features)【38†L112-L117】.  
- **Offline:** Enable offline use similarly (Drive offline mode【25†L49-L57】). On mobile, files can be made available offline.  
- **Limits:** Converted (imported) Slides decks have a 100 MB file size limit【11†L57-L60】. A Slides presentation can have thousands of slides, but in practice performance degrades with very large decks. The Slides editor auto-saves and keeps revisions【38†L123-L127】.  

【38†L86-L90】【38†L136-L141】

---

## Platform-Specific Clients

- **Web (drive.google.com + docs.google.com, sheets.google.com, slides.google.com).** The web UIs are the fullest-featured. Users can upload any files; create new Docs/Sheets/Slides; preview and annotate PDFs/images; search across Drive; install add-ons; manage Trash; and adjust settings (e.g. offline mode, storage). Web allows third-party app integrations (e.g. add-ons, Google Apps Script). In Chrome/Edge, enabling “Offline” in Drive settings lets you open recent Docs/Sheets/Slides without internet【25†L49-L57】.  

- **Desktop (Drive for Desktop app).** This provides a **native sync client** on Windows/macOS. It mounts Drive as a virtual drive (or a mirrored folder). Key features【1†L54-L61】【30†L192-L199】【31†L31-L39】:  
  - *Streaming vs. Mirroring:* Stream mode keeps files in cloud (downloads on demand); mirror mode keeps a full local copy of My Drive. Either way, edits sync both ways【31†L31-L39】.  
  - *Integration:* Files open with local apps. Google Docs/Sheets/Slides files open in the browser; Office/Adobe files open in their native desktop apps.  
  - *File Access:* You can use desktop search (Explorer/Finder) to find Drive files. The app itself has a search bar that queries the cloud【30†L192-L201】.  
  - *Offline:* In Mirror mode, all files are offline-ready. In Stream mode, you can mark specific files for offline use.  
  - *Multiple Accounts:* Supports up to 4 Google accounts signed in simultaneously【1†L144-L150】.  
  - *Backup:* Lets you back up local folders (Desktop, Documents, Pictures) to Drive and optionally to Google Photos.  

- **Mobile (Android & iOS apps).** There are separate mobile apps for Drive, Docs, Sheets, Slides. In Drive app:  
  - Browse/search your Drive and shared drives. Upload photos/files or scan paper docs to PDF【13†L615-L619】.  
  - Preview files and photos; share links or invite via Google contacts.  
  - Keep files offline by long-press/tap “Make available offline.” There’s an Offline view to see these files.  
  - Manage file permissions, star, move, rename, etc.  
  - Receive push notifications for shared file activity or comments.  
  - **Scan documents:** Drive app can scan receipts or documents with the camera into OCR’d PDF files【13†L615-L619】.  
  - **Editing:** To edit, Drive offers buttons that open the file in the corresponding Docs/Sheets/Slides app. The native editors on mobile support most common editing features (formatting text, inserting images, adding charts, etc.), though the interface is simplified and some advanced features (like Apps Script, add-ons, complex layout) are omitted.  

---

## Security, Administration & Integration

- **Security & Privacy.** All data is encrypted in transit (TLS) and at rest with Google’s keys by default【35†L171-L174】【37†L153-L156】【38†L154-L157】. Google Workspace offers Client-Side Encryption (CSE) options for extra data control. Two-factor authentication (2FA) can be enforced on accounts. Google’s zero-trust infrastructure and malware scanning protect Drive files.  
- **Access Control & IRM.** As above, Drive/Docs support fine-grained sharing. Info Rights Management can disable download/print/copy for any user role (viewer, commenter, and now even editors)【23†L89-L98】. Admins can set sharing policies (e.g. block external shares, restrict domain sharing, turn off download).  
- **Data Loss Prevention (DLP).** Workspace admins can define DLP rules: automatically detect and label sensitive content in Drive (SSNs, credit cards, etc.) and restrict sharing. Google introduced AI-powered content classification to auto-label Drive files in 2025【5†L299-L307】.  
- **Audit & Compliance.** Admins have logs (Drive Audit) to see file activity (views, edits, shares). Google Vault can archive Drive files for legal hold/eDiscovery. Drive is GDPR, HIPAA, and ISO certified (various compliance badges【35†L171-L179】【37†L153-L156】【38†L154-L157】).  
- **Third-Party Apps & APIs:** Google Drive has an API for custom apps. Workspace Marketplace offers dozens of integrations (e.g. DocuSign, Slack). In-Drive Add-ons (via the “+New” > “More” menu) let you launch apps that can create or edit Drive files. Drive is also accessible via Google Workspace apps (Calendar, Chat, Gmail).  

---

## Technical Limits & Notes

- **File & Folder Limits:** No hard limit on number of files/folders, but extremely large Drive trees may slow performance. Each Team Shared Drive can contain **at least 400,000** items per drive (Drive API limit).  
- **Storage Quotas:** Free Drive: 15 GB total. Google One plans: 100 GB, 200 GB, 2 TB, etc. Workspace Business Starter: 30 GB; Business Standard: 2 TB; Business Plus/Enterprise: 5 TB or unlimited (depending on plan)【13†L599-L602】.  
- **Uploading:** Maximum file upload size is 5 TB【11†L73-L76】【15†L981-L986】. However, Google Workspace enforces a 750 GB/day upload limit per user【15†L981-L986】. (If you hit 750 GB in 24 hrs, uploads pause.) You can upload folders or drag-drop multiple files via web.  
- **Versioning:** Google Docs (native) has no version limit besides size. Non-Google files store up to 100 revisions (or 30 days).  
- **Offline:** Drive for Desktop mirror mode provides offline working; web offline requires Chrome/Edge and enabling offline access【25†L49-L57】. Mobile apps allow offline caching of specific files.  

---

**Sources:** Official Google documentation and release notes have been used to compile this feature breakdown【11†L41-L49】【13†L599-L602】【23†L89-L98】【25†L49-L57】【35†L101-L104】【37†L85-L89】【38†L86-L90】【15†L981-L986】. Each bullet above is informed by these sources, reflecting Google’s publicly documented features as of 2026.

