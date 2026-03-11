'use client';

import React, { useEffect, useRef } from 'react';
import { Info, Pencil, Star, StarOff, Download, Trash2, Link } from 'lucide-react';
import { type FileItem } from '@/lib/api';
import styles from './FileContextMenu.module.css';

interface Props {
  file: FileItem;
  x: number;
  y: number;
  onClose: () => void;
  onInfo: () => void;
  onRename: () => void;
  onStarToggle: () => void;
  onDownload: () => void;
  onDelete: () => void;
  onCopyLink: () => void;
}

export function FileContextMenu({
  file,
  x,
  y,
  onClose,
  onInfo,
  onRename,
  onStarToggle,
  onDownload,
  onDelete,
  onCopyLink,
}: Props) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function onMouseDown(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    }
    function onKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') onClose();
    }
    document.addEventListener('mousedown', onMouseDown);
    document.addEventListener('keydown', onKeyDown);
    return () => {
      document.removeEventListener('mousedown', onMouseDown);
      document.removeEventListener('keydown', onKeyDown);
    };
  }, [onClose]);

  // Adjust position to stay within viewport
  const adjustedX = Math.min(x, window.innerWidth - 200);
  const adjustedY = Math.min(y, window.innerHeight - 280);

  const items = [
    { icon: <Info size={14} />, label: 'File info', action: onInfo },
    { icon: <Pencil size={14} />, label: 'Rename', action: onRename },
    {
      icon: file.isStarred ? <StarOff size={14} /> : <Star size={14} />,
      label: file.isStarred ? 'Remove star' : 'Star',
      action: onStarToggle,
    },
    { icon: <Link size={14} />, label: 'Copy link', action: onCopyLink },
    { icon: <Download size={14} />, label: 'Download', action: onDownload },
    null,
    { icon: <Trash2 size={14} />, label: 'Move to trash', action: onDelete, danger: true },
  ] as const;

  return (
    <div
      ref={ref}
      className={styles.menu}
      style={{ left: adjustedX, top: adjustedY }}
      role="menu"
      aria-label="File options"
    >
      {items.map((item, i) =>
        item === null ? (
          <div key={i} className={styles.separator} role="separator" />
        ) : (
          <button
            key={i}
            type="button"
            className={[styles.item, 'danger' in item && item.danger ? styles.danger : '']
              .filter(Boolean)
              .join(' ')}
            role="menuitem"
            onClick={() => {
              item.action();
              onClose();
            }}
          >
            <span className={styles.itemIcon}>{item.icon}</span>
            {item.label}
          </button>
        )
      )}
    </div>
  );
}
