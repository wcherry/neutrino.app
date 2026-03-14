'use client';

import React from 'react';
import type { Editor } from '@tiptap/react';
import styles from './page.module.css';

interface OutlineItem {
  level: number;
  text: string;
  id: string;
}

interface DocOutlineProps {
  editor: Editor | null;
}

function getOutlineItems(editor: Editor): OutlineItem[] {
  const items: OutlineItem[] = [];
  editor.state.doc.descendants((node, pos) => {
    if (node.type.name.startsWith('heading')) {
      const level = node.attrs.level as number;
      const text = node.textContent;
      if (text.trim()) {
        items.push({ level, text, id: `heading-${pos}` });
      }
    }
  });
  return items;
}

function scrollToHeading(editor: Editor, text: string, level: number) {
  editor.state.doc.descendants((node, pos) => {
    if (
      node.type.name.startsWith('heading') &&
      node.attrs.level === level &&
      node.textContent === text
    ) {
      const domNode = editor.view.nodeDOM(pos);
      if (domNode && domNode instanceof HTMLElement) {
        domNode.scrollIntoView({ behavior: 'smooth', block: 'start' });
      }
      return false;
    }
  });
}

export function DocOutline({ editor }: DocOutlineProps) {
  if (!editor) return null;

  const items = getOutlineItems(editor);

  if (items.length === 0) {
    return (
      <div className={styles.outlinePanel}>
        <div className={styles.outlineTitle}>Outline</div>
        <div style={{ padding: '8px 16px', fontSize: 12, color: 'var(--color-text-muted, #5f6368)' }}>
          Add headings to see an outline.
        </div>
      </div>
    );
  }

  const indentClass: Record<number, string> = {
    1: '',
    2: styles.outlineItemH2,
    3: styles.outlineItemH3,
    4: styles.outlineItemH4,
  };

  return (
    <div className={styles.outlinePanel}>
      <div className={styles.outlineTitle}>Outline</div>
      {items.map((item, idx) => (
        <button
          key={idx}
          className={`${styles.outlineItem} ${indentClass[Math.min(item.level, 4)] ?? styles.outlineItemH4}`}
          onClick={() => scrollToHeading(editor, item.text, item.level)}
          title={item.text}
        >
          {item.text}
        </button>
      ))}
    </div>
  );
}
