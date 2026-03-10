'use client';

import React, { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  Heading,
  Text,
  Button,
  Card,
  Badge,
  Breadcrumbs,
  EmptyState,
  FileListSkeleton,
  Skeleton,
} from '@neutrino/ui';
import {
  Upload,
  FolderPlus,
  LayoutGrid,
  Grid3x3,
  List,
  AlignJustify,
  Folder,
  FileText,
  FileImage,
  FileVideo,
  Music,
  Archive,
  File,
  MoreVertical,
  Clock,
  Star,
} from 'lucide-react';
import { storageApi, type FileItem } from '@/lib/api';
import { UploadZone } from './UploadZone';
import { PreviewModal } from './PreviewModal';
import styles from './page.module.css';

type ViewMode = 'large' | 'small' | 'list';

function getFileIcon(mimeType: string) {
  if (mimeType.startsWith('image/')) return FileImage;
  if (mimeType.startsWith('video/')) return FileVideo;
  if (mimeType.startsWith('audio/')) return Music;
  if (mimeType.includes('zip') || mimeType.includes('tar') || mimeType.includes('rar')) return Archive;
  if (mimeType.includes('text') || mimeType.includes('document')) return FileText;
  return File;
}

function getIconColor(mimeType: string): string {
  if (mimeType.startsWith('image/')) return 'var(--color-violet, #7c3aed)';
  if (mimeType.startsWith('video/')) return 'var(--color-rose, #e11d48)';
  if (mimeType.startsWith('audio/')) return 'var(--color-amber, #d97706)';
  if (mimeType.includes('zip') || mimeType.includes('tar')) return 'var(--color-orange, #ea580c)';
  if (mimeType.includes('text') || mimeType.includes('document')) return 'var(--color-accent)';
  return 'var(--color-text-muted)';
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

const MOCK_RECENT_FILES: FileItem[] = [
  {
    id: 'f1',
    name: 'Q4-Report.pdf',
    sizeBytes: 4200000,
    mimeType: 'application/pdf',
    folderId: null,
    isStarred: false,
    createdAt: '2026-03-07T10:00:00Z',
    updatedAt: '2026-03-07T10:00:00Z',
  },
  {
    id: 'f2',
    name: 'design-mockup.png',
    sizeBytes: 2100000,
    mimeType: 'image/png',
    folderId: null,
    isStarred: false,
    createdAt: '2026-03-06T15:30:00Z',
    updatedAt: '2026-03-06T15:30:00Z',
  },
  {
    id: 'f3',
    name: 'team-meeting-notes.txt',
    sizeBytes: 8500,
    mimeType: 'text/plain',
    folderId: null,
    isStarred: false,
    createdAt: '2026-03-05T09:00:00Z',
    updatedAt: '2026-03-05T09:00:00Z',
  },
];

const VIEW_BUTTONS: { mode: ViewMode; icon: React.ReactNode; label: string }[] = [
  { mode: 'large', icon: <LayoutGrid size={15} />, label: 'Large grid' },
  { mode: 'small', icon: <Grid3x3 size={15} />, label: 'Small grid' },
  { mode: 'list',  icon: <AlignJustify size={15} />, label: 'Detailed list' },
];

export default function DrivePage() {
  const [viewMode, setViewMode] = useState<ViewMode>('large');
  const [uploadOpen, setUploadOpen] = useState(false);
  const [previewFile, setPreviewFile] = useState<FileItem | null>(null);

  const { data: filesData, isLoading, isError } = useQuery({
    queryKey: ['files', { limit: 50, offset: 0 }],
    queryFn: () => storageApi.listFiles({ limit: 50, offset: 0 }),
    placeholderData: { items: MOCK_RECENT_FILES, total: MOCK_RECENT_FILES.length, page: 1, pageSize: 50, totalPages: 1 },
  });

  const files = filesData?.items ?? [];

  return (
    <div
      className={styles.page}
      onDragOver={(e) => { e.preventDefault(); setUploadOpen(true); }}
    >
      {/* Page header */}
      <div className={styles.header}>
        <div className={styles['header-left']}>
          <Breadcrumbs items={[{ label: 'My Drive' }]} />
          <Heading level={1} size="xl">My Drive</Heading>
        </div>
        <div className={styles['header-actions']}>
          <Button variant="ghost" size="sm" icon={<FolderPlus size={16} />}>
            New folder
          </Button>
          <Button variant="primary" size="sm" icon={<Upload size={16} />} onClick={() => setUploadOpen(true)}>
            Upload
          </Button>
          <div className={styles['view-toggle']} role="group" aria-label="View mode">
            {VIEW_BUTTONS.map(({ mode, icon, label }) => (
              <button
                key={mode}
                type="button"
                className={[styles['view-btn'], viewMode === mode ? styles['view-btn-active'] : ''].filter(Boolean).join(' ')}
                onClick={() => setViewMode(mode)}
                aria-label={label}
                aria-pressed={viewMode === mode}
                title={label}
              >
                {icon}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Quick access */}
      <section className={styles.section} aria-labelledby="quick-access-heading">
        <div className={styles['section-header']}>
          <Heading level={2} size="sm" id="quick-access-heading">Quick access</Heading>
          <Text as="span" size="xs" color="muted">
            <Clock size={12} style={{ display: 'inline', marginRight: '4px', verticalAlign: 'middle' }} />
            Recently viewed
          </Text>
        </div>
        <div className={styles['quick-grid']}>
          {isLoading
            ? Array.from({ length: 4 }, (_, i) => (
                <div key={i} className={styles['quick-card-skeleton']}>
                  <Skeleton shape="rect" width={32} height={32} />
                  <div style={{ flex: 1 }}>
                    <Skeleton shape="text" width="80%" height="0.875rem" />
                    <Skeleton shape="text" width="50%" height="0.75rem" style={{ marginTop: '4px' }} />
                  </div>
                </div>
              ))
            : MOCK_RECENT_FILES.slice(0, 4).map((file) => {
                const IconComponent = getFileIcon(file.mimeType);
                return (
                  <Card key={file.id} hoverable padding="sm" className={styles['quick-card']} role="button" tabIndex={0} aria-label={`Open ${file.name}`}>
                    <div className={styles['quick-card-inner']}>
                      <div className={styles['file-icon-sm']} style={{ color: getIconColor(file.mimeType) }}>
                        <IconComponent size={20} strokeWidth={1.5} />
                      </div>
                      <div className={styles['quick-card-info']}>
                        <Text size="sm" weight="medium" truncate>{file.name}</Text>
                        <Text size="xs" color="muted">{formatDate(file.updatedAt)}</Text>
                      </div>
                    </div>
                  </Card>
                );
              })}
        </div>
      </section>

      {/* All files */}
      <section className={styles.section} aria-labelledby="all-files-heading">
        <div className={styles['section-header']}>
          <Heading level={2} size="sm" id="all-files-heading">Files</Heading>
          {filesData && (
            <Badge variant="default" size="sm">{filesData.total} items</Badge>
          )}
        </div>

        {isLoading ? (
          <div className={styles['list-container']}>
            <FileListSkeleton rows={8} />
          </div>
        ) : isError ? (
          <EmptyState
            title="Could not load files"
            description="There was an error loading your files. Please try again."
            action={<Button variant="secondary" size="sm" onClick={() => window.location.reload()}>Retry</Button>}
          />
        ) : files.length === 0 ? (
          <EmptyState
            icon={Folder}
            title="No files yet"
            description="Upload files to get started. Your files will appear here."
            action={
              <Button variant="primary" size="sm" icon={<Upload size={16} />} onClick={() => setUploadOpen(true)}>
                Upload your first file
              </Button>
            }
          />
        ) : viewMode === 'large' ? (
          /* ── Large grid ── */
          <div className={styles['grid-large']} role="list">
            {files.map((file) => {
              const IconComponent = getFileIcon(file.mimeType);
              return (
                <Card key={file.id} hoverable padding="none" className={styles['card-large']} role="listitem" tabIndex={0} aria-label={file.name} onClick={() => setPreviewFile(file)}>
                  <div className={styles['preview-large']} style={{ color: getIconColor(file.mimeType) }}>
                    <IconComponent size={48} strokeWidth={1} />
                  </div>
                  <div className={styles['card-large-body']}>
                    <Text size="sm" weight="medium" truncate>{file.name}</Text>
                    <Text size="xs" color="muted">{formatFileSize(file.sizeBytes)}</Text>
                  </div>
                  <button type="button" className={styles['file-menu-btn']} aria-label={`More options for ${file.name}`} onClick={(e) => e.stopPropagation()}>
                    <MoreVertical size={14} />
                  </button>
                </Card>
              );
            })}
          </div>
        ) : viewMode === 'small' ? (
          /* ── Small grid ── */
          <div className={styles['grid-small']} role="list">
            {files.map((file) => {
              const IconComponent = getFileIcon(file.mimeType);
              return (
                <Card key={file.id} hoverable padding="none" className={styles['card-small']} role="listitem" tabIndex={0} aria-label={file.name} onClick={() => setPreviewFile(file)}>
                  <div className={styles['preview-small']} style={{ color: getIconColor(file.mimeType) }}>
                    <IconComponent size={28} strokeWidth={1.25} />
                  </div>
                  <div className={styles['card-small-body']}>
                    <Text size="xs" weight="medium" truncate>{file.name}</Text>
                  </div>
                  <button type="button" className={styles['file-menu-btn']} aria-label={`More options for ${file.name}`} onClick={(e) => e.stopPropagation()}>
                    <MoreVertical size={12} />
                  </button>
                </Card>
              );
            })}
          </div>
        ) : (
          /* ── Detailed list ── */
          <div className={styles['list-container']}>
            <div className={styles['list-header']}>
              <Text size="xs" color="muted" weight="semibold">Name</Text>
              <Text size="xs" color="muted" weight="semibold">Type</Text>
              <Text size="xs" color="muted" weight="semibold">Size</Text>
              <Text size="xs" color="muted" weight="semibold">Modified</Text>
              <span />
            </div>
            <div role="list">
              {files.map((file) => {
                const IconComponent = getFileIcon(file.mimeType);
                const ext = file.name.includes('.') ? file.name.split('.').pop()!.toUpperCase() : '—';
                return (
                  <div key={file.id} className={styles['list-row']} role="listitem" tabIndex={0} aria-label={file.name} onClick={() => setPreviewFile(file)}>
                    <div className={styles['list-name']}>
                      <span className={styles['file-icon-sm']} style={{ color: getIconColor(file.mimeType) }}>
                        <IconComponent size={18} strokeWidth={1.5} />
                      </span>
                      <Text size="sm" truncate>{file.name}</Text>
                      {file.isStarred && <Star size={12} style={{ color: 'var(--color-amber, #d97706)', flexShrink: 0 }} />}
                    </div>
                    <Text size="sm" color="muted">{ext}</Text>
                    <Text size="sm" color="muted">{formatFileSize(file.sizeBytes)}</Text>
                    <Text size="sm" color="muted">{formatDate(file.updatedAt)}</Text>
                    <button type="button" className={styles['file-menu-btn']} aria-label={`More options for ${file.name}`}>
                      <MoreVertical size={14} />
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </section>

      {uploadOpen && <UploadZone onClose={() => setUploadOpen(false)} />}
      {previewFile && <PreviewModal file={previewFile} onClose={() => setPreviewFile(null)} />}
    </div>
  );
}
