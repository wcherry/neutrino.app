'use client';

import React, { useState, useEffect, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { Menu, ChevronRight } from 'lucide-react';
import type { Editor } from '@tiptap/react';
import styles from './MenuBar.module.css';

// ── Help modal ────────────────────────────────────────────────────────────

const SHORTCUTS = [
  { action: 'Bold',             keys: ['Ctrl', 'B'] },
  { action: 'Italic',           keys: ['Ctrl', 'I'] },
  { action: 'Underline',        keys: ['Ctrl', 'U'] },
  { action: 'Strikethrough',    keys: ['Alt', 'Shift', '5'] },
  { action: 'Clear formatting', keys: ['Ctrl', '\\'] },
  { action: 'Undo',             keys: ['Ctrl', 'Z'] },
  { action: 'Redo',             keys: ['Ctrl', 'Y'] },
  { action: 'Save',             keys: ['Ctrl', 'S'] },
  { action: 'Select all',       keys: ['Ctrl', 'A'] },
  { action: 'Find in page',     keys: ['Ctrl', 'F'] },
  { action: 'Insert link',      keys: ['Ctrl', 'K'] },
  { action: 'Print',            keys: ['Ctrl', 'P'] },
  { action: 'Heading 1',        keys: ['Ctrl', 'Alt', '1'] },
  { action: 'Heading 2',        keys: ['Ctrl', 'Alt', '2'] },
  { action: 'Heading 3',        keys: ['Ctrl', 'Alt', '3'] },
  { action: 'Normal text',      keys: ['Ctrl', 'Alt', '0'] },
  { action: 'Bullet list',      keys: ['Ctrl', 'Shift', '8'] },
  { action: 'Numbered list',    keys: ['Ctrl', 'Shift', '7'] },
];

function HelpModal({ onClose }: { onClose: () => void }) {
  return (
    <div className={styles.modalOverlay} onClick={onClose}>
      <div className={styles.helpModal} onClick={e => e.stopPropagation()}>
        <div className={styles.helpHeader}>
          <span className={styles.helpTitle}>Neutrino Docs — Help</span>
          <button className={styles.helpClose} onClick={onClose}>✕</button>
        </div>

        <div className={styles.helpBody}>
          <section className={styles.helpSection}>
            <h3 className={styles.helpSectionTitle}>Getting started</h3>
            <ul className={styles.helpList}>
              <li>Click anywhere on the page to start typing.</li>
              <li>Use the toolbar to format text, insert tables, links, and images.</li>
              <li>Documents save automatically — look for "All changes saved" in the top bar.</li>
              <li>Use <strong>File → Page setup</strong> to change paper size, orientation, and margins.</li>
              <li>Use <strong>File → Export</strong> to download as Word, PDF, HTML, or plain text.</li>
            </ul>
          </section>

          <section className={styles.helpSection}>
            <h3 className={styles.helpSectionTitle}>Keyboard shortcuts</h3>
            <div className={styles.shortcutsGrid}>
              {SHORTCUTS.map(({ action, keys }) => (
                <div key={action} className={styles.shortcutRow}>
                  <span className={styles.shortcutAction}>{action}</span>
                  <span className={styles.shortcutKeys}>
                    {keys.map((k, i) => (
                      <React.Fragment key={k}>
                        {i > 0 && <span className={styles.shortcutPlus}>+</span>}
                        <kbd className={styles.kbd}>{k}</kbd>
                      </React.Fragment>
                    ))}
                  </span>
                </div>
              ))}
            </div>
          </section>

          <section className={styles.helpSection}>
            <h3 className={styles.helpSectionTitle}>Tips</h3>
            <ul className={styles.helpList}>
              <li>Right-click anywhere in the document for a context menu.</li>
              <li>Use the Outline panel to navigate long documents by heading.</li>
              <li>Version History lets you restore any previous save.</li>
              <li>Comments let you annotate specific sections.</li>
              <li>Import a Word (.docx) file to convert it to a Neutrino Doc.</li>
            </ul>
          </section>

          <section className={styles.helpSection}>
            <h3 className={styles.helpSectionTitle}>About</h3>
            <p className={styles.helpAbout}>
              Neutrino Docs is part of the Neutrino productivity suite — a Google Workspace-compatible
              platform for documents, spreadsheets, and cloud storage.
            </p>
          </section>
        </div>
      </div>
    </div>
  );
}

// ── Menu item types ───────────────────────────────────────────────────────

type MenuItem =
  | { kind: 'action';    label: string; shortcut?: string; disabled?: boolean; action: () => void }
  | { kind: 'separator' }
  | { kind: 'submenu';   label: string; items: MenuItem[] };

// ── Recursive panel item ──────────────────────────────────────────────────

function PanelItem({ item }: { item: MenuItem }) {
  const [subOpen, setSubOpen] = useState(false);

  if (item.kind === 'separator') {
    return <div className={styles.panelSep} />;
  }

  if (item.kind === 'submenu') {
    return (
      <div
        className={`${styles.panelItem} ${styles.hasSubmenu}`}
        onMouseEnter={() => setSubOpen(true)}
        onMouseLeave={() => setSubOpen(false)}
      >
        <span>{item.label}</span>
        <ChevronRight size={13} className={styles.submenuArrow} />
        {subOpen && (
          <div className={styles.submenuPanel}>
            {item.items.map((child, i) => (
              <PanelItem key={i} item={child} />
            ))}
          </div>
        )}
      </div>
    );
  }

  return (
    <button
      className={styles.panelItem}
      disabled={item.disabled}
      onClick={item.action}
    >
      <span>{item.label}</span>
      {item.shortcut && <span className={styles.panelShortcut}>{item.shortcut}</span>}
    </button>
  );
}

// ── HamburgerMenu ─────────────────────────────────────────────────────────

export interface HamburgerMenuProps {
  editor: Editor | null;
  titleInputRef: React.RefObject<HTMLInputElement>;
  onSave: () => void;
  onNewDoc: () => void;
  onDuplicate: () => void;
  onImport: () => void;
  onExport: (format: 'docx' | 'pdf' | 'html' | 'txt') => void;
  onPageSetup: () => void;
  onPrint: () => void;
}

export function HamburgerMenu({
  editor,
  titleInputRef,
  onSave,
  onNewDoc,
  onDuplicate,
  onImport,
  onExport,
  onPageSetup,
  onPrint,
}: HamburgerMenuProps) {
  const router = useRouter();
  const [open, setOpen] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const wrapperRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (wrapperRef.current && !wrapperRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [open]);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        onSave();
      }
    };
    document.addEventListener('keydown', handler);
    return () => document.removeEventListener('keydown', handler);
  }, [onSave]);

  const close = () => setOpen(false);

  const menuItems: MenuItem[] = [
    {
      kind: 'submenu',
      label: 'File',
      items: [
        { kind: 'action',  label: 'New document',  shortcut: 'Ctrl+N', action: () => { close(); onNewDoc(); } },
        { kind: 'action',  label: 'Open docs list',                    action: () => { close(); router.push('/docs'); } },
        { kind: 'separator' },
        { kind: 'action',  label: 'Rename',                            action: () => { close(); titleInputRef.current?.focus(); titleInputRef.current?.select(); } },
        { kind: 'action',  label: 'Duplicate',                         action: () => { close(); onDuplicate(); } },
        { kind: 'separator' },
        { kind: 'action',  label: 'Save',          shortcut: 'Ctrl+S', action: () => { close(); onSave(); } },
        { kind: 'separator' },
        { kind: 'action',  label: 'Import (.docx)',                    action: () => { close(); onImport(); } },
        {
          kind: 'submenu', label: 'Export as…', items: [
            { kind: 'action', label: 'Microsoft Word (.docx)', action: () => { close(); onExport('docx'); } },
            { kind: 'action', label: 'PDF (.pdf)',             action: () => { close(); onExport('pdf'); } },
            { kind: 'action', label: 'Web page (.html)',       action: () => { close(); onExport('html'); } },
            { kind: 'action', label: 'Plain text (.txt)',      action: () => { close(); onExport('txt'); } },
          ],
        },
        { kind: 'separator' },
        { kind: 'action',  label: 'Page setup…',                       action: () => { close(); onPageSetup(); } },
        { kind: 'action',  label: 'Print…',        shortcut: 'Ctrl+P', action: () => { close(); onPrint(); } },
      ],
    },
    {
      kind: 'submenu',
      label: 'Edit',
      items: [
        { kind: 'action', label: 'Undo',       shortcut: 'Ctrl+Z', disabled: !editor?.can().undo(), action: () => { close(); editor?.chain().focus().undo().run(); } },
        { kind: 'action', label: 'Redo',       shortcut: 'Ctrl+Y', disabled: !editor?.can().redo(), action: () => { close(); editor?.chain().focus().redo().run(); } },
        { kind: 'separator' },
        { kind: 'action', label: 'Cut',        shortcut: 'Ctrl+X', action: () => { close(); document.execCommand('cut'); } },
        { kind: 'action', label: 'Copy',       shortcut: 'Ctrl+C', action: () => { close(); document.execCommand('copy'); } },
        { kind: 'action', label: 'Paste',      shortcut: 'Ctrl+V', action: () => { close(); document.execCommand('paste'); } },
        { kind: 'separator' },
        { kind: 'action', label: 'Select all', shortcut: 'Ctrl+A', action: () => { close(); editor?.chain().focus().selectAll().run(); } },
      ],
    },
    {
      kind: 'submenu',
      label: 'Format',
      items: [
        { kind: 'action', label: 'Bold',         shortcut: 'Ctrl+B',     action: () => { close(); editor?.chain().focus().toggleBold().run(); } },
        { kind: 'action', label: 'Italic',       shortcut: 'Ctrl+I',     action: () => { close(); editor?.chain().focus().toggleItalic().run(); } },
        { kind: 'action', label: 'Underline',    shortcut: 'Ctrl+U',     action: () => { close(); editor?.chain().focus().toggleUnderline().run(); } },
        { kind: 'action', label: 'Strikethrough',                        action: () => { close(); editor?.chain().focus().toggleStrike().run(); } },
        { kind: 'separator' },
        { kind: 'action', label: 'Clear formatting', shortcut: 'Ctrl+\\', action: () => { close(); editor?.chain().focus().clearNodes().unsetAllMarks().run(); } },
        { kind: 'separator' },
        {
          kind: 'submenu', label: 'Paragraph style', items: [
            { kind: 'action', label: 'Normal text', shortcut: 'Ctrl+Alt+0', action: () => { close(); editor?.chain().focus().setParagraph().run(); } },
            { kind: 'separator' },
            { kind: 'action', label: 'Heading 1',  shortcut: 'Ctrl+Alt+1', action: () => { close(); editor?.chain().focus().toggleHeading({ level: 1 }).run(); } },
            { kind: 'action', label: 'Heading 2',  shortcut: 'Ctrl+Alt+2', action: () => { close(); editor?.chain().focus().toggleHeading({ level: 2 }).run(); } },
            { kind: 'action', label: 'Heading 3',  shortcut: 'Ctrl+Alt+3', action: () => { close(); editor?.chain().focus().toggleHeading({ level: 3 }).run(); } },
            { kind: 'action', label: 'Heading 4',                          action: () => { close(); editor?.chain().focus().toggleHeading({ level: 4 }).run(); } },
            { kind: 'action', label: 'Heading 5',                          action: () => { close(); editor?.chain().focus().toggleHeading({ level: 5 }).run(); } },
            { kind: 'action', label: 'Heading 6',                          action: () => { close(); editor?.chain().focus().toggleHeading({ level: 6 }).run(); } },
          ],
        },
        {
          kind: 'submenu', label: 'Line spacing', items: [
            { kind: 'action', label: 'Single (1.0)',      action: () => { close(); } },
            { kind: 'action', label: 'Comfortable (1.15)', action: () => { close(); } },
            { kind: 'action', label: 'Double (2.0)',      action: () => { close(); } },
          ],
        },
      ],
    },
    {
      kind: 'submenu',
      label: 'Insert',
      items: [
        { kind: 'action', label: 'Link…',           shortcut: 'Ctrl+K', action: () => { close(); const url = window.prompt('Enter URL:', 'https://'); if (url) editor?.chain().focus().setLink({ href: url }).run(); } },
        { kind: 'action', label: 'Image…',                              action: () => { close(); const url = window.prompt('Enter image URL:'); if (url) editor?.chain().focus().setImage({ src: url }).run(); } },
        { kind: 'separator' },
        { kind: 'action', label: 'Table',                               action: () => { close(); editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(); } },
        { kind: 'action', label: 'Horizontal rule',                     action: () => { close(); editor?.chain().focus().setHorizontalRule().run(); } },
        { kind: 'action', label: 'Code block',                          action: () => { close(); editor?.chain().focus().toggleCodeBlock().run(); } },
        { kind: 'action', label: 'Blockquote',                          action: () => { close(); editor?.chain().focus().toggleBlockquote().run(); } },
      ],
    },
    {
      kind: 'submenu',
      label: 'Help',
      items: [
        { kind: 'action', label: 'Keyboard shortcuts & help', action: () => { close(); setShowHelp(true); } },
      ],
    },
  ];

  return (
    <>
      <div ref={wrapperRef} className={styles.hamburgerWrapper}>
        <button
          className={`${styles.hamburgerBtn} ${open ? styles.hamburgerBtnOpen : ''}`}
          onClick={() => setOpen(v => !v)}
          title="Menu"
          aria-label="Open menu"
        >
          <Menu size={18} />
        </button>

        {open && (
          <div className={styles.panel}>
            {menuItems.map((item, i) => (
              <PanelItem key={i} item={item} />
            ))}
          </div>
        )}
      </div>

      {showHelp && <HelpModal onClose={() => setShowHelp(false)} />}
    </>
  );
}
