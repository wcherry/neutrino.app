'use client';

import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import TextStyle from '@tiptap/extension-text-style';
import { Color } from '@tiptap/extension-color';
import FontFamily from '@tiptap/extension-font-family';
import TextAlign from '@tiptap/extension-text-align';
import Table from '@tiptap/extension-table';
import TableRow from '@tiptap/extension-table-row';
import TableCell from '@tiptap/extension-table-cell';
import TableHeader from '@tiptap/extension-table-header';
import Image from '@tiptap/extension-image';
import Link from '@tiptap/extension-link';
import Placeholder from '@tiptap/extension-placeholder';
import CharacterCount from '@tiptap/extension-character-count';
import Highlight from '@tiptap/extension-highlight';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  ArrowLeft, FileText, History, MessageSquare,
} from 'lucide-react';
import { docsApi, driveReadContent, driveWriteContent, type PageSetup } from '@/lib/api';
import { Toolbar } from './Toolbar';
import { HamburgerMenu } from './MenuBar';
import { DocOutline } from './DocOutline';
import { VersionHistoryPanel } from '@/components/VersionHistoryPanel';
import { CommentsPanel } from '@/components/CommentsPanel';
import { EditorContextMenu } from './EditorContextMenu';
import styles from './page.module.css';

// ── Paper sizes ───────────────────────────────────────────────────────────
// Dimensions in inches; rendered at 96 dpi for screen display.

const PAPER_SIZES: Record<PageSetup['pageSize'], { w: number; h: number }> = {
  letter:    { w: 8.5,   h: 11 },
  a4:        { w: 8.27,  h: 11.69 },
  legal:     { w: 8.5,   h: 14 },
  a3:        { w: 11.69, h: 16.54 },
  a5:        { w: 5.83,  h: 8.27 },
  tabloid:   { w: 11,    h: 17 },
  executive: { w: 7.25,  h: 10.5 },
};

const SCREEN_DPI = 96;

function pageDimensions(ps: PageSetup): { widthPx: number; heightPx: number } {
  const size = PAPER_SIZES[ps.pageSize] ?? PAPER_SIZES.letter;
  const wPx = size.w * SCREEN_DPI;
  const hPx = size.h * SCREEN_DPI;
  return ps.orientation === 'landscape'
    ? { widthPx: hPx, heightPx: wPx }
    : { widthPx: wPx, heightPx: hPx };
}

// ── Page-break overlay ────────────────────────────────────────────────────
// Renders horizontal lines inside the page div at every content-height interval.

interface PageBreakOverlayProps {
  marginTop: number;
  contentHeightPx: number;
  pageRef: React.RefObject<HTMLDivElement>;
}

function PageBreakOverlay({ marginTop, contentHeightPx, pageRef }: PageBreakOverlayProps) {
  const [pageHeight, setPageHeight] = useState(0);

  useEffect(() => {
    const el = pageRef.current;
    if (!el) return;
    const obs = new ResizeObserver(() => setPageHeight(el.scrollHeight));
    obs.observe(el);
    return () => obs.disconnect();
  }, [pageRef]);

  if (contentHeightPx <= 0) return null;

  const count = Math.floor((pageHeight - marginTop) / contentHeightPx);
  return (
    <>
      {Array.from({ length: Math.max(0, count) }, (_, i) => (
        <div
          key={i}
          className={styles.pageBreakLine}
          style={{ top: marginTop + (i + 1) * contentHeightPx }}
        />
      ))}
    </>
  );
}

// ── Print ──────────────────────────────────────────────────────────────────

function printDoc(title: string, html: string, ps: PageSetup) {
  const pw = window.open('', '_blank');
  if (!pw) return;

  const pageSizeMap: Record<string, string> = {
    letter: 'letter', a4: 'A4', legal: 'legal',
    a3: 'A3', a5: 'A5', tabloid: 'tabloid', executive: '7.25in 10.5in',
  };
  const cssSize = `${pageSizeMap[ps.pageSize] ?? 'letter'} ${ps.orientation}`;
  // Margins stored as pt; keep them in pt for @page
  const m = `${ps.marginTop}pt ${ps.marginRight}pt ${ps.marginBottom}pt ${ps.marginLeft}pt`;

  pw.document.write(`<!DOCTYPE html><html><head>
<meta charset="UTF-8"><title>${title}</title>
<style>
@page { size: ${cssSize}; margin: ${m}; }
* { box-sizing: border-box; }
body { margin: 0; font-family: Arial, sans-serif; font-size: 11pt; color: #000; }
h1 { font-size: 20pt; margin: 0 0 12pt; }
h2 { font-size: 16pt; margin: 0 0 10pt; }
h3 { font-size: 13pt; margin: 0 0 8pt; }
h4, h5, h6 { font-size: 11pt; margin: 0 0 8pt; }
p { margin: 0 0 8pt; line-height: 1.5; }
table { border-collapse: collapse; width: 100%; margin: 8pt 0; }
td, th { border: 1px solid #ccc; padding: 4pt 8pt; }
th { background: #f0f0f0; font-weight: bold; }
img { max-width: 100%; }
code { font-family: monospace; background: #f5f5f5; padding: 1pt 4pt; border-radius: 2pt; }
pre { background: #f5f5f5; padding: 8pt; margin: 8pt 0; font-family: monospace; white-space: pre-wrap; }
blockquote { margin: 8pt 0 8pt 24pt; border-left: 3px solid #ccc; padding-left: 12pt; color: #555; }
ul, ol { margin: 0 0 8pt; padding-left: 24pt; }
a { color: #1a73e8; }
strong { font-weight: bold; }
em { font-style: italic; }
u { text-decoration: underline; }
s { text-decoration: line-through; }
mark { background: #fff176; }
hr { border: none; border-top: 1px solid #ccc; margin: 12pt 0; }
</style></head><body>${html}</body></html>`);
  pw.document.close();
  pw.focus();
  setTimeout(() => { pw.print(); pw.close(); }, 300);
}

// ── DOCX / PDF export helpers ──────────────────────────────────────────────

async function exportAsDocx(title: string, html: string) {
  const { Document, Packer, Paragraph, TextRun } = await import('docx');
  const { saveAs } = await import('file-saver');

  const lines = html.replace(/<[^>]+>/g, '\n').split('\n').filter(l => l.trim());
  const paras = lines.map(l => new Paragraph({ children: [new TextRun(l.trim())] }));

  const doc = new Document({ sections: [{ children: paras }] });
  const blob = await Packer.toBlob(doc);
  saveAs(blob, `${title}.docx`);
}

function exportAsHtml(title: string, html: string) {
  const full = `<!DOCTYPE html><html><head><meta charset="UTF-8"><title>${title}</title></head><body>${html}</body></html>`;
  const blob = new Blob([full], { type: 'text/html' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url; a.download = `${title}.html`; a.click();
  URL.revokeObjectURL(url);
}

async function exportAsTxt(title: string, docId: string) {
  const result = await docsApi.exportText(docId);
  const blob = new Blob([result.text], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url; a.download = `${title}.txt`; a.click();
  URL.revokeObjectURL(url);
}

// ── Page setup modal ────────────────────────────────────────────────────────

interface PageSetupModalProps {
  pageSetup: PageSetup;
  onSave: (ps: PageSetup) => void;
  onClose: () => void;
}

function PageSetupModal({ pageSetup, onSave, onClose }: PageSetupModalProps) {
  const [ps, setPs] = useState<PageSetup>(pageSetup);

  return (
    <div className={styles.modalOverlay} onClick={onClose}>
      <div className={styles.modal} onClick={e => e.stopPropagation()}>
        <div className={styles.modalTitle}>Page setup</div>

        <div className={styles.formRow}>
          <label className={styles.formLabel}>Page size</label>
          <select className={styles.formSelect} value={ps.pageSize}
            onChange={e => setPs(p => ({ ...p, pageSize: e.target.value as PageSetup['pageSize'] }))}>
            <option value="letter">Letter (8.5" × 11")</option>
            <option value="a4">A4 (8.27" × 11.69")</option>
            <option value="legal">Legal (8.5" × 14")</option>
            <option value="a3">A3 (11.69" × 16.54")</option>
            <option value="a5">A5 (5.83" × 8.27")</option>
            <option value="tabloid">Tabloid (11" × 17")</option>
            <option value="executive">Executive (7.25" × 10.5")</option>
          </select>
        </div>

        <div className={styles.formRow}>
          <label className={styles.formLabel}>Orientation</label>
          <select className={styles.formSelect} value={ps.orientation}
            onChange={e => setPs(p => ({ ...p, orientation: e.target.value as PageSetup['orientation'] }))}>
            <option value="portrait">Portrait</option>
            <option value="landscape">Landscape</option>
          </select>
        </div>

        <div className={styles.formRow}>
          <label className={styles.formLabel}>Top margin (pt)</label>
          <input className={styles.formInput} type="number" value={ps.marginTop}
            onChange={e => setPs(p => ({ ...p, marginTop: Number(e.target.value) }))} />
        </div>
        <div className={styles.formRow}>
          <label className={styles.formLabel}>Bottom margin (pt)</label>
          <input className={styles.formInput} type="number" value={ps.marginBottom}
            onChange={e => setPs(p => ({ ...p, marginBottom: Number(e.target.value) }))} />
        </div>
        <div className={styles.formRow}>
          <label className={styles.formLabel}>Left margin (pt)</label>
          <input className={styles.formInput} type="number" value={ps.marginLeft}
            onChange={e => setPs(p => ({ ...p, marginLeft: Number(e.target.value) }))} />
        </div>
        <div className={styles.formRow}>
          <label className={styles.formLabel}>Right margin (pt)</label>
          <input className={styles.formInput} type="number" value={ps.marginRight}
            onChange={e => setPs(p => ({ ...p, marginRight: Number(e.target.value) }))} />
        </div>

        <div className={styles.modalActions}>
          <button className={styles.exportBtn} onClick={onClose}>Cancel</button>
          <button className={styles.exportBtn}
            style={{ background: '#1a73e8', color: 'white', border: 'none' }}
            onClick={() => { onSave(ps); onClose(); }}>
            Apply
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Main editor ──────────────────────────────────────────────────────────────

const AUTO_SAVE_DELAY_MS = 2000;

export function DocEditor() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const docId = searchParams.get('id') ?? '';
  const queryClient = useQueryClient();

  const [title, setTitle] = useState('');
  const [pageSetup, setPageSetup] = useState<PageSetup>({
    marginTop: 72, marginBottom: 72, marginLeft: 72, marginRight: 72,
    orientation: 'portrait', pageSize: 'letter',
  });
  const [saveStatus, setSaveStatus] = useState<'saved' | 'saving' | 'unsaved'>('saved');
  const [showPageSetup, setShowPageSetup] = useState(false);
  const [showOutline, setShowOutline] = useState(true);
  const [showHistory, setShowHistory] = useState(false);
  const [showComments, setShowComments] = useState(false);
  const [commentInitialText, setCommentInitialText] = useState('');
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number } | null>(null);
  const importInputRef = useRef<HTMLInputElement>(null);
  const titleInputRef = useRef<HTMLInputElement>(null);
  const autoSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingContent = useRef<string | null>(null);
  const pageRef = useRef<HTMLDivElement>(null);

  const { data: doc, isLoading: metaLoading } = useQuery({
    queryKey: ['doc', docId],
    queryFn: () => docsApi.getDoc(docId),
    staleTime: 0,
    enabled: !!docId,
  });

  const { data: docContent, isLoading: contentLoading } = useQuery({
    queryKey: ['doc-content', docId],
    queryFn: () => driveReadContent(doc!.contentUrl),
    enabled: !!doc?.contentUrl,
    staleTime: 0,
  });

  const isLoading = metaLoading || contentLoading;

  const contentMutation = useMutation({
    mutationFn: (content: string) =>
      driveWriteContent(doc!.contentWriteUrl, content, 'doc.json'),
    onMutate: () => setSaveStatus('saving'),
    onSuccess: () => {
      setSaveStatus('saved');
      queryClient.invalidateQueries({ queryKey: ['folder-contents'] });
    },
    onError: () => setSaveStatus('unsaved'),
  });

  const metaMutation = useMutation({
    mutationFn: (body: Parameters<typeof docsApi.saveDoc>[1]) =>
      docsApi.saveDoc(docId, body),
  });

  const triggerSave = useCallback(
    (content: string) => {
      contentMutation.mutate(content);
    },
    [contentMutation]
  );

  const editor = useEditor({
    extensions: [
      StarterKit,
      Underline,
      TextStyle,
      Color,
      FontFamily,
      Highlight.configure({ multicolor: true }),
      TextAlign.configure({ types: ['heading', 'paragraph'] }),
      Table.configure({ resizable: true }),
      TableRow,
      TableCell,
      TableHeader,
      Image.configure({ inline: true, allowBase64: true }),
      Link.configure({ openOnClick: false }),
      Placeholder.configure({ placeholder: 'Start typing…' }),
      CharacterCount,
    ],
    editorProps: {
      attributes: { class: 'ProseMirror', spellcheck: 'true' },
    },
    onUpdate: ({ editor }) => {
      const content = JSON.stringify(editor.getJSON());
      pendingContent.current = content;
      setSaveStatus('unsaved');
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
      autoSaveTimer.current = setTimeout(() => {
        triggerSave(content);
      }, AUTO_SAVE_DELAY_MS);
    },
  });

  useEffect(() => {
    if (!doc || !editor) return;
    setTitle(doc.title);
    setPageSetup(doc.pageSetup);
  }, [doc, editor]);

  useEffect(() => {
    if (!docContent || !editor) return;
    try {
      const json = JSON.parse(docContent);
      editor.commands.setContent(json, false);
    } catch {
      editor.commands.setContent(docContent, false);
    }
  }, [docContent, editor]);

  useEffect(() => {
    return () => {
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    };
  }, []);

  const handleTitleBlur = () => {
    if (!title.trim() || title === doc?.title) return;
    metaMutation.mutate({ title });
  };

  const handleManualSave = useCallback(() => {
    if (!editor) return;
    const content = JSON.stringify(editor.getJSON());
    contentMutation.mutate(content);
  }, [editor, contentMutation]);

  const handleNewDoc = useCallback(async () => {
    const newDoc = await docsApi.createDoc({ title: 'Untitled document' });
    router.push(`/docs/editor?id=${newDoc.id}`);
  }, [router]);

  const handleDuplicate = useCallback(async () => {
    if (!editor || !doc) return;
    const newDoc = await docsApi.createDoc({ title: `${title} (copy)` });
    const content = JSON.stringify(editor.getJSON());
    await driveWriteContent(newDoc.contentWriteUrl, content, 'doc.json');
    router.push(`/docs/editor?id=${newDoc.id}`);
  }, [editor, doc, title, router]);

  const handlePageSetupSave = (ps: PageSetup) => {
    setPageSetup(ps);
    metaMutation.mutate({ pageSetup: ps });
  };

  const handleInsertImage = () => {
    const url = window.prompt('Enter image URL:');
    if (url && editor) {
      editor.chain().focus().setImage({ src: url }).run();
    }
  };

  const handleInsertLink = () => {
    const url = window.prompt('Enter URL:');
    if (url && editor) {
      editor.chain().focus().setLink({ href: url }).run();
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    setContextMenu({ x: e.clientX, y: e.clientY });
  };

  const handleAddComment = (selectedText: string) => {
    setCommentInitialText(selectedText ? `"${selectedText}"\n\n` : '');
    setShowComments(true);
  };

  const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !editor) return;
    const { convertToHtml } = await import('mammoth');
    const arrayBuffer = await file.arrayBuffer();
    const result = await convertToHtml({ arrayBuffer });
    editor.commands.setContent(result.value, true);
  };

  const handlePrint = useCallback(() => {
    if (!editor) return;
    printDoc(title, editor.getHTML(), pageSetup);
  }, [editor, title, pageSetup]);

  const handleExport = async (format: string) => {
    if (!editor) return;
    const html = editor.getHTML();
    if (format === 'pdf') {
      printDoc(title, html, pageSetup);
    } else if (format === 'html') {
      exportAsHtml(title, html);
    } else if (format === 'txt') {
      await exportAsTxt(title, docId);
    } else if (format === 'docx') {
      await exportAsDocx(title, html);
    }
  };

  const wordCount = editor ? editor.storage.characterCount.words() : 0;
  const charCount = editor ? editor.storage.characterCount.characters() : 0;

  const { widthPx, heightPx } = useMemo(() => pageDimensions(pageSetup), [pageSetup]);
  const contentHeightPx = heightPx - pageSetup.marginTop - pageSetup.marginBottom;

  const pageStyle: React.CSSProperties = {
    width: widthPx,
    paddingTop: pageSetup.marginTop,
    paddingBottom: pageSetup.marginBottom,
    paddingLeft: pageSetup.marginLeft,
    paddingRight: pageSetup.marginRight,
  };

  if (isLoading || !docId) {
    return <div className={styles.loading}>Loading document…</div>;
  }

  return (
    <div className={styles.shell}>
      {/* ── Top bar ── */}
      <div className={styles.topbar}>
        <HamburgerMenu
          editor={editor}
          titleInputRef={titleInputRef}
          onSave={handleManualSave}
          onNewDoc={handleNewDoc}
          onDuplicate={handleDuplicate}
          onImport={() => importInputRef.current?.click()}
          onExport={handleExport}
          onPageSetup={() => setShowPageSetup(true)}
          onPrint={handlePrint}
        />

        <button className={styles.backBtn} onClick={() => router.push('/docs')}>
          <ArrowLeft size={16} />
          Docs
        </button>

        <div className={styles.docIcon}>
          <FileText size={18} />
        </div>

        <input
          ref={titleInputRef}
          className={styles.titleInput}
          value={title}
          onChange={e => setTitle(e.target.value)}
          onBlur={handleTitleBlur}
          placeholder="Untitled document"
          spellCheck={false}
        />

        <div className={styles.topbarActions}>
          <span className={styles.saveStatus}>
            {saveStatus === 'saving' ? 'Saving…' : saveStatus === 'unsaved' ? 'Unsaved changes' : 'All changes saved'}
          </span>

          <input
            ref={importInputRef}
            type="file"
            accept=".docx"
            className={styles.hiddenInput}
            onChange={handleImport}
          />

          <button
            className={styles.exportBtn}
            onClick={() => setShowOutline(v => !v)}
            title="Toggle outline"
            style={{ opacity: showOutline ? 1 : 0.5 }}
          >
            ≡ Outline
          </button>

          <button
            className={styles.exportBtn}
            onClick={() => setShowHistory(v => !v)}
            title="Version history"
            style={{ opacity: showHistory ? 1 : 0.5 }}
          >
            <History size={14} /> History
          </button>

          <button
            className={styles.exportBtn}
            onClick={() => setShowComments(v => !v)}
            title="Comments"
            style={{ opacity: showComments ? 1 : 0.5 }}
          >
            <MessageSquare size={14} /> Comments
          </button>
        </div>
      </div>

      {/* ── Toolbar ── */}
      <Toolbar editor={editor} onInsertImage={handleInsertImage} />

      {/* ── Main area ── */}
      <div className={styles.mainArea}>
        {showOutline && <DocOutline editor={editor} />}
        <div className={styles.editorScroll} onContextMenu={handleContextMenu}>
          <div ref={pageRef} className={styles.page} style={pageStyle}>
            <PageBreakOverlay
              marginTop={pageSetup.marginTop}
              contentHeightPx={contentHeightPx}
              pageRef={pageRef}
            />
            <div className={styles.editorContent}>
              <EditorContent editor={editor} />
            </div>
          </div>
        </div>
        {showHistory && (
          <VersionHistoryPanel
            fileId={docId}
            onRestore={() => {
              queryClient.invalidateQueries({ queryKey: ['doc-content', docId] });
            }}
            onClose={() => setShowHistory(false)}
          />
        )}
        {showComments && (
          <CommentsPanel
            fileId={docId}
            onClose={() => { setShowComments(false); setCommentInitialText(''); }}
            initialText={commentInitialText}
          />
        )}
      </div>

      {/* ── Status bar ── */}
      <div className={styles.statusBar}>
        <span>{wordCount.toLocaleString()} words</span>
        <span>{charCount.toLocaleString()} characters</span>
        {charCount > 1_020_000 && (
          <span style={{ color: '#d93025' }}>
            ⚠ Approaching 1M character limit ({charCount.toLocaleString()} / 1,020,000)
          </span>
        )}
      </div>

      {showPageSetup && (
        <PageSetupModal pageSetup={pageSetup} onSave={handlePageSetupSave} onClose={() => setShowPageSetup(false)} />
      )}

      {contextMenu && editor && (
        <EditorContextMenu
          editor={editor}
          x={contextMenu.x}
          y={contextMenu.y}
          hasSelection={!editor.state.selection.empty}
          onClose={() => setContextMenu(null)}
          onAddComment={handleAddComment}
          onInsertLink={handleInsertLink}
        />
      )}
    </div>
  );
}
