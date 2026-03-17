'use client';

import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { storageApi, type FileVersionItem } from '@/lib/api';
import { History, RotateCcw, Tag, X, Check } from 'lucide-react';
import styles from './VersionHistoryPanel.module.css';

interface VersionHistoryPanelProps {
  fileId: string;
  onRestore?: () => void;
  onClose: () => void;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function VersionHistoryPanel({ fileId, onRestore, onClose }: VersionHistoryPanelProps) {
  const queryClient = useQueryClient();
  const [editingLabel, setEditingLabel] = useState<string | null>(null);
  const [labelInput, setLabelInput] = useState('');
  const [restoringId, setRestoringId] = useState<string | null>(null);

  const { data, isLoading, isError } = useQuery({
    queryKey: ['versions', fileId],
    queryFn: () => storageApi.listVersions(fileId),
    staleTime: 10_000,
  });

  const labelMutation = useMutation({
    mutationFn: ({ versionId, label }: { versionId: string; label: string }) =>
      storageApi.labelVersion(fileId, versionId, label),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['versions', fileId] });
      setEditingLabel(null);
    },
  });

  const restoreMutation = useMutation({
    mutationFn: (versionId: string) => storageApi.restoreVersion(fileId, versionId),
    onSuccess: () => {
      setRestoringId(null);
      queryClient.invalidateQueries({ queryKey: ['versions', fileId] });
      onRestore?.();
    },
    onError: () => setRestoringId(null),
  });

  function startLabel(v: FileVersionItem) {
    setEditingLabel(v.id);
    setLabelInput(v.label ?? '');
  }

  function submitLabel(versionId: string) {
    if (labelInput.trim()) {
      labelMutation.mutate({ versionId, label: labelInput.trim() });
    } else {
      setEditingLabel(null);
    }
  }

  function handleRestore(versionId: string) {
    setRestoringId(versionId);
    restoreMutation.mutate(versionId);
  }

  const versions = data?.versions ?? [];

  return (
    <div className={styles.panel}>
      <div className={styles.header}>
        <div className={styles.headerTitle}>
          <History size={16} />
          Version history
        </div>
        <button className={styles.closeBtn} onClick={onClose} title="Close">
          <X size={16} />
        </button>
      </div>

      <div className={styles.list}>
        {isLoading && (
          <div className={styles.empty}>Loading versions…</div>
        )}
        {isError && (
          <div className={styles.empty} style={{ color: 'var(--color-danger, #dc2626)' }}>
            Failed to load version history.
          </div>
        )}
        {!isLoading && !isError && versions.length === 0 && (
          <div className={styles.empty}>No versions yet.</div>
        )}
        {versions.map((v, idx) => (
          <div key={v.id} className={styles.versionRow}>
            <div className={styles.versionMeta}>
              <div className={styles.versionNum}>
                v{v.versionNumber}
                {idx === 0 && <span className={styles.currentBadge}>Current</span>}
              </div>
              <div className={styles.versionDate}>{formatDate(v.createdAt)}</div>
              <div className={styles.versionSize}>{formatBytes(v.sizeBytes)}</div>
            </div>

            {editingLabel === v.id ? (
              <div className={styles.labelRow}>
                <input
                  className={styles.labelInput}
                  value={labelInput}
                  onChange={e => setLabelInput(e.target.value)}
                  onKeyDown={e => {
                    if (e.key === 'Enter') submitLabel(v.id);
                    if (e.key === 'Escape') setEditingLabel(null);
                  }}
                  autoFocus
                  placeholder="Version name…"
                />
                <button
                  className={styles.iconBtn}
                  onClick={() => submitLabel(v.id)}
                  disabled={labelMutation.isPending}
                >
                  <Check size={13} />
                </button>
                <button className={styles.iconBtn} onClick={() => setEditingLabel(null)}>
                  <X size={13} />
                </button>
              </div>
            ) : (
              <div className={styles.labelRow}>
                {v.label && <span className={styles.labelBadge}>{v.label}</span>}
                <button
                  className={styles.actionBtn}
                  onClick={() => startLabel(v)}
                  title="Name this version"
                >
                  <Tag size={12} />
                  {v.label ? 'Rename' : 'Name'}
                </button>
                {idx !== 0 && (
                  <button
                    className={styles.actionBtn}
                    onClick={() => handleRestore(v.id)}
                    disabled={restoringId === v.id || restoreMutation.isPending}
                    title="Restore this version"
                  >
                    <RotateCcw size={12} />
                    {restoringId === v.id ? 'Restoring…' : 'Restore'}
                  </button>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
