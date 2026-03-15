'use client';

import React, { useState, useEffect, useRef, useCallback } from 'react';
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
  ArrowLeft, FileText, Download, Upload, ChevronDown, Settings,
} from 'lucide-react';
import { docsApi, type PageSetup } from '@/lib/api';
import { Toolbar } from './Toolbar';
import { DocOutline } from './DocOutline';
import styles from './page.module.css';

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
  const [showExportMenu, setShowExportMenu] = useState(false);
  const [showPageSetup, setShowPageSetup] = useState(false);
  const [showOutline, setShowOutline] = useState(true);
  const importInputRef = useRef<HTMLInputElement>(null);
  const autoSaveTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pendingContent = useRef<string | null>(null);

  const { data: doc, isLoading } = useQuery({
    queryKey: ['doc', docId],
    queryFn: () => docsApi.getDoc(docId),
    staleTime: 0,
    enabled: !!docId,
  });

  const saveMutation = useMutation({
    mutationFn: (body: Parameters<typeof docsApi.saveDoc>[1]) =>
      docsApi.saveDoc(docId, body),
    onMutate: () => setSaveStatus('saving'),
    onSuccess: () => {
      setSaveStatus('saved');
      queryClient.invalidateQueries({ queryKey: ['folder-contents'] });
    },
    onError: () => setSaveStatus('unsaved'),
  });

  const triggerSave = useCallback(
    (content: string, currentTitle: string, currentPageSetup: PageSetup) => {
      saveMutation.mutate({ content, title: currentTitle, pageSetup: currentPageSetup });
    },
    [saveMutation]
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
        triggerSave(content, title, pageSetup);
      }, AUTO_SAVE_DELAY_MS);
    },
  });

  useEffect(() => {
    if (!doc || !editor) return;
    setTitle(doc.title);
    setPageSetup(doc.pageSetup);
    try {
      const json = JSON.parse(doc.content);
      editor.commands.setContent(json, false);
    } catch {
      editor.commands.setContent(doc.content, false);
    }
  }, [doc, editor]);

  useEffect(() => {
    return () => {
      if (autoSaveTimer.current) clearTimeout(autoSaveTimer.current);
    };
  }, []);

  const handleTitleBlur = () => {
    if (!title.trim() || title === doc?.title) return;
    const content = pendingContent.current ?? doc?.content ?? '';
    triggerSave(content, title, pageSetup);
  };

  const handlePageSetupSave = (ps: PageSetup) => {
    setPageSetup(ps);
    const content = pendingContent.current ?? doc?.content ?? '';
    triggerSave(content, title, ps);
  };

  const handleInsertImage = () => {
    const url = window.prompt('Enter image URL:');
    if (url && editor) {
      editor.chain().focus().setImage({ src: url }).run();
    }
  };

  const handleImport = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !editor) return;
    const { convertToHtml } = await import('mammoth');
    const arrayBuffer = await file.arrayBuffer();
    const result = await convertToHtml({ arrayBuffer });
    editor.commands.setContent(result.value, true);
  };

  const handleExport = async (format: string) => {
    setShowExportMenu(false);
    if (!editor) return;
    const html = editor.getHTML();
    if (format === 'pdf') {
      window.print();
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

  const pagePaddingStyle: React.CSSProperties = {
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
        <button className={styles.backBtn} onClick={() => router.push('/docs')}>
          <ArrowLeft size={16} />
          Docs
        </button>

        <div className={styles.docIcon}>
          <FileText size={18} />
        </div>

        <input
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

          <button className={styles.exportBtn} onClick={() => importInputRef.current?.click()} title="Import DOCX">
            <Upload size={14} /> Import
          </button>
          <input
            ref={importInputRef}
            type="file"
            accept=".docx"
            className={styles.hiddenInput}
            onChange={handleImport}
          />

          <div style={{ position: 'relative' }}>
            <button className={styles.exportBtn} onClick={() => setShowExportMenu(v => !v)}>
              <Download size={14} /> Export <ChevronDown size={12} />
            </button>
            {showExportMenu && (
              <div className={styles.exportMenu}>
                <button className={styles.exportMenuItem} onClick={() => handleExport('docx')}>Microsoft Word (.docx)</button>
                <button className={styles.exportMenuItem} onClick={() => handleExport('pdf')}>PDF (.pdf)</button>
                <button className={styles.exportMenuItem} onClick={() => handleExport('html')}>Web page (.html)</button>
                <button className={styles.exportMenuItem} onClick={() => handleExport('txt')}>Plain text (.txt)</button>
              </div>
            )}
          </div>

          <button className={styles.exportBtn} onClick={() => setShowPageSetup(true)} title="Page setup">
            <Settings size={14} />
          </button>

          <button
            className={styles.exportBtn}
            onClick={() => setShowOutline(v => !v)}
            title="Toggle outline"
            style={{ opacity: showOutline ? 1 : 0.5 }}
          >
            ≡ Outline
          </button>
        </div>
      </div>

      {/* ── Toolbar ── */}
      <Toolbar editor={editor} onInsertImage={handleInsertImage} />

      {/* ── Main area ── */}
      <div className={styles.mainArea}>
        {showOutline && <DocOutline editor={editor} />}
        <div className={styles.editorScroll}>
          <div className={styles.page} style={pagePaddingStyle}>
            <div className={styles.editorContent}>
              <EditorContent editor={editor} />
            </div>
          </div>
        </div>
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

      {showExportMenu && (
        <div style={{ position: 'fixed', inset: 0, zIndex: 99 }} onClick={() => setShowExportMenu(false)} />
      )}
    </div>
  );
}
