'use client';

import React, { useRef } from 'react';
import type { Editor } from '@tiptap/react';
import {
  Bold, Italic, Underline, Strikethrough, Code,
  List, ListOrdered, AlignLeft, AlignCenter, AlignRight, AlignJustify,
  Link, Image, Table, Minus, Undo, Redo, Quote,
} from 'lucide-react';
import styles from './page.module.css';

interface ToolbarProps {
  editor: Editor | null;
  onInsertImage: () => void;
}

const FONT_FAMILIES = [
  { label: 'Default', value: '' },
  { label: 'Arial', value: 'Arial, sans-serif' },
  { label: 'Georgia', value: 'Georgia, serif' },
  { label: 'Times New Roman', value: "'Times New Roman', serif" },
  { label: 'Courier New', value: "'Courier New', monospace" },
  { label: 'Verdana', value: 'Verdana, sans-serif' },
  { label: 'Trebuchet MS', value: "'Trebuchet MS', sans-serif" },
];

const FONT_SIZES = ['8', '9', '10', '11', '12', '14', '16', '18', '20', '24', '28', '32', '36', '48', '60', '72'];

const HEADINGS: { label: string; level: number | null }[] = [
  { label: 'Normal', level: null },
  { label: 'Heading 1', level: 1 },
  { label: 'Heading 2', level: 2 },
  { label: 'Heading 3', level: 3 },
  { label: 'Heading 4', level: 4 },
  { label: 'Heading 5', level: 5 },
  { label: 'Heading 6', level: 6 },
];

const TEXT_COLORS = ['#202124', '#d93025', '#e37400', '#0f9d58', '#1a73e8', '#a142f4', '#f538a0'];
const HIGHLIGHT_COLORS = ['transparent', '#fef08a', '#bbf7d0', '#bae6fd', '#e9d5ff', '#fecaca'];

export function Toolbar({ editor, onInsertImage }: ToolbarProps) {
  const colorInputRef = useRef<HTMLInputElement>(null);
  const highlightInputRef = useRef<HTMLInputElement>(null);

  if (!editor) return null;

  const currentHeading = HEADINGS.find(h =>
    h.level ? editor.isActive('heading', { level: h.level }) : !editor.isActive('heading')
  );

  const setLink = () => {
    const previous = editor.getAttributes('link').href as string | undefined;
    const url = window.prompt('Enter URL:', previous ?? 'https://');
    if (url === null) return;
    if (url === '') {
      editor.chain().focus().extendMarkRange('link').unsetLink().run();
    } else {
      editor.chain().focus().extendMarkRange('link').setLink({ href: url }).run();
    }
  };

  const insertTable = () => {
    editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
  };

  return (
    <div className={styles.toolbar}>
      {/* Undo/Redo */}
      <div className={styles.toolbarGroup}>
        <button
          className={styles.toolBtn}
          onClick={() => editor.chain().focus().undo().run()}
          disabled={!editor.can().undo()}
          title="Undo (Ctrl+Z)"
        >
          <Undo size={15} />
        </button>
        <button
          className={styles.toolBtn}
          onClick={() => editor.chain().focus().redo().run()}
          disabled={!editor.can().redo()}
          title="Redo (Ctrl+Y)"
        >
          <Redo size={15} />
        </button>
      </div>

      <div className={styles.toolbarDivider} />

      {/* Paragraph / heading style */}
      <select
        className={styles.toolbarSelect}
        value={currentHeading?.label ?? 'Normal'}
        onChange={e => {
          const item = HEADINGS.find(h => h.label === e.target.value);
          if (!item) return;
          if (item.level === null) {
            editor.chain().focus().setParagraph().run();
          } else {
            editor.chain().focus().toggleHeading({ level: item.level as 1|2|3|4|5|6 }).run();
          }
        }}
        title="Paragraph style"
        style={{ width: 110 }}
      >
        {HEADINGS.map(h => (
          <option key={h.label} value={h.label}>{h.label}</option>
        ))}
      </select>

      <div className={styles.toolbarDivider} />

      {/* Font family */}
      <select
        className={styles.toolbarSelect}
        value={editor.getAttributes('textStyle').fontFamily ?? ''}
        onChange={e => {
          if (e.target.value === '') {
            editor.chain().focus().unsetFontFamily().run();
          } else {
            editor.chain().focus().setFontFamily(e.target.value).run();
          }
        }}
        title="Font family"
        style={{ width: 120 }}
      >
        {FONT_FAMILIES.map(f => (
          <option key={f.value} value={f.value}>{f.label}</option>
        ))}
      </select>

      {/* Font size */}
      <select
        className={styles.toolbarSelect}
        style={{ width: 56 }}
        title="Font size"
        defaultValue="11"
        onChange={e => {
          editor.chain().focus().setMark('textStyle', { fontSize: `${e.target.value}pt` }).run();
        }}
      >
        {FONT_SIZES.map(s => (
          <option key={s} value={s}>{s}</option>
        ))}
      </select>

      <div className={styles.toolbarDivider} />

      {/* Bold / Italic / Underline / Strikethrough */}
      <div className={styles.toolbarGroup}>
        <button
          className={`${styles.toolBtn} ${editor.isActive('bold') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleBold().run()}
          title="Bold (Ctrl+B)"
          style={{ fontWeight: 700 }}
        >B</button>
        <button
          className={`${styles.toolBtn} ${editor.isActive('italic') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleItalic().run()}
          title="Italic (Ctrl+I)"
          style={{ fontStyle: 'italic' }}
        >I</button>
        <button
          className={`${styles.toolBtn} ${editor.isActive('underline') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleUnderline().run()}
          title="Underline (Ctrl+U)"
          style={{ textDecoration: 'underline' }}
        >U</button>
        <button
          className={`${styles.toolBtn} ${editor.isActive('strike') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleStrike().run()}
          title="Strikethrough"
          style={{ textDecoration: 'line-through' }}
        >S</button>
      </div>

      <div className={styles.toolbarDivider} />

      {/* Text color / Highlight */}
      <div className={styles.toolbarGroup}>
        <button
          className={styles.toolBtn}
          onClick={() => colorInputRef.current?.click()}
          title="Text color"
          style={{ flexDirection: 'column', gap: 1, height: 32 }}
        >
          <span style={{ fontSize: 13, fontWeight: 600, lineHeight: 1 }}>A</span>
          <span
            className={styles.colorSwatch}
            style={{ background: editor.getAttributes('textStyle').color ?? '#202124' }}
          />
          <input
            ref={colorInputRef}
            type="color"
            className={styles.hiddenInput}
            defaultValue="#202124"
            onChange={e => editor.chain().focus().setColor(e.target.value).run()}
          />
        </button>
        <button
          className={styles.toolBtn}
          onClick={() => highlightInputRef.current?.click()}
          title="Highlight color"
          style={{ flexDirection: 'column', gap: 1, height: 32 }}
        >
          <span style={{ fontSize: 12, lineHeight: 1 }}>⬛</span>
          <span
            className={styles.colorSwatch}
            style={{ background: editor.getAttributes('highlight').color ?? '#fef08a' }}
          />
          <input
            ref={highlightInputRef}
            type="color"
            className={styles.hiddenInput}
            defaultValue="#fef08a"
            onChange={e => editor.chain().focus().toggleHighlight({ color: e.target.value }).run()}
          />
        </button>
      </div>

      <div className={styles.toolbarDivider} />

      {/* Text align */}
      <div className={styles.toolbarGroup}>
        <button
          className={`${styles.toolBtn} ${editor.isActive({ textAlign: 'left' }) ? styles.active : ''}`}
          onClick={() => editor.chain().focus().setTextAlign('left').run()}
          title="Align left"
        ><AlignLeft size={15} /></button>
        <button
          className={`${styles.toolBtn} ${editor.isActive({ textAlign: 'center' }) ? styles.active : ''}`}
          onClick={() => editor.chain().focus().setTextAlign('center').run()}
          title="Align center"
        ><AlignCenter size={15} /></button>
        <button
          className={`${styles.toolBtn} ${editor.isActive({ textAlign: 'right' }) ? styles.active : ''}`}
          onClick={() => editor.chain().focus().setTextAlign('right').run()}
          title="Align right"
        ><AlignRight size={15} /></button>
        <button
          className={`${styles.toolBtn} ${editor.isActive({ textAlign: 'justify' }) ? styles.active : ''}`}
          onClick={() => editor.chain().focus().setTextAlign('justify').run()}
          title="Justify"
        ><AlignJustify size={15} /></button>
      </div>

      <div className={styles.toolbarDivider} />

      {/* Lists */}
      <div className={styles.toolbarGroup}>
        <button
          className={`${styles.toolBtn} ${editor.isActive('bulletList') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleBulletList().run()}
          title="Bulleted list"
        ><List size={15} /></button>
        <button
          className={`${styles.toolBtn} ${editor.isActive('orderedList') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleOrderedList().run()}
          title="Numbered list"
        ><ListOrdered size={15} /></button>
        <button
          className={`${styles.toolBtn} ${editor.isActive('blockquote') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleBlockquote().run()}
          title="Quote"
        ><Quote size={15} /></button>
        <button
          className={`${styles.toolBtn} ${editor.isActive('code') ? styles.active : ''}`}
          onClick={() => editor.chain().focus().toggleCode().run()}
          title="Inline code"
        ><Code size={15} /></button>
      </div>

      <div className={styles.toolbarDivider} />

      {/* Insert */}
      <div className={styles.toolbarGroup}>
        <button
          className={`${styles.toolBtn} ${editor.isActive('link') ? styles.active : ''}`}
          onClick={setLink}
          title="Insert link"
        ><Link size={15} /></button>
        <button
          className={styles.toolBtn}
          onClick={onInsertImage}
          title="Insert image"
        ><Image size={15} /></button>
        <button
          className={styles.toolBtn}
          onClick={insertTable}
          title="Insert table"
        ><Table size={15} /></button>
        <button
          className={styles.toolBtn}
          onClick={() => editor.chain().focus().setHorizontalRule().run()}
          title="Horizontal rule"
        ><Minus size={15} /></button>
      </div>

      {/* Table controls (visible when in table) */}
      {editor.isActive('table') && (
        <>
          <div className={styles.toolbarDivider} />
          <div className={styles.toolbarGroup}>
            <button
              className={styles.toolBtnWide}
              onClick={() => editor.chain().focus().addColumnAfter().run()}
              title="Add column"
            >+Col</button>
            <button
              className={styles.toolBtnWide}
              onClick={() => editor.chain().focus().addRowAfter().run()}
              title="Add row"
            >+Row</button>
            <button
              className={styles.toolBtnWide}
              onClick={() => editor.chain().focus().deleteColumn().run()}
              title="Delete column"
            >-Col</button>
            <button
              className={styles.toolBtnWide}
              onClick={() => editor.chain().focus().deleteRow().run()}
              title="Delete row"
            >-Row</button>
            <button
              className={styles.toolBtnWide}
              onClick={() => editor.chain().focus().mergeCells().run()}
              title="Merge cells"
            >Merge</button>
            <button
              className={styles.toolBtnWide}
              onClick={() => editor.chain().focus().splitCell().run()}
              title="Split cell"
            >Split</button>
            <button
              className={styles.toolBtnWide}
              onClick={() => editor.chain().focus().deleteTable().run()}
              title="Delete table"
              style={{ color: '#d93025' }}
            >Del Table</button>
          </div>
        </>
      )}
    </div>
  );
}
