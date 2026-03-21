'use client';

import React, { useState, useRef, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Heading,
  Text,
  Button,
  Card,
  Breadcrumbs,
  EmptyState,
  Skeleton,
  useToast,
} from '@neutrino/ui';
import {
  Upload,
  FolderPlus,
  FileText,
  Folder,
  FileImage,
  FileVideo,
  Music,
  Archive,
  File,
  Clock,
} from 'lucide-react';
import { storageApi, filesystemApi, docsApi, sheetsApi, slidesApi, type FileItem, type Folder as FolderItem } from '@/lib/api';
import { useRouter } from 'next/navigation';
import { UploadZone } from './UploadZone';
import { PreviewModal } from './PreviewModal';
import { FileContextMenu } from './FileContextMenu';
import { FileInfoPanel } from './FileInfoPanel';
import { ShareDialog } from './ShareDialog';
import { FileGrid, type GridItem, type SortField, type SortDir } from '@/components/FileGrid';
import styles from './page.module.css';

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

function folderToGridItem(folder: FolderItem): GridItem {
  return {
    id: folder.id,
    name: folder.name,
    kind: 'folder',
    icon: Folder,
    iconColor: folder.color ?? 'var(--color-amber, #d97706)',
    subtitle: 'Folder',
    typeText: 'Folder',
    sizeText: '—',
    modifiedText: formatDate(folder.updatedAt),
    isStarred: folder.isStarred,
  };
}

function fileToGridItem(file: FileItem): GridItem {
  const ext = file.name.includes('.') ? file.name.split('.').pop()!.toUpperCase() : '—';
  return {
    id: file.id,
    name: file.name,
    kind: 'file',
    icon: getFileIcon(file.mimeType),
    iconColor: getIconColor(file.mimeType),
    subtitle: formatFileSize(file.sizeBytes),
    mimeType: file.mimeType,
    typeText: ext,
    sizeText: formatFileSize(file.sizeBytes),
    modifiedText: formatDate(file.updatedAt),
    isStarred: file.isStarred,
    coverThumbnail: file.coverThumbnail,
    coverThumbnailMimeType: file.coverThumbnailMimeType,
  };
}

const DOC_MIME = 'application/x-neutrino-doc';
const SHEET_MIME = 'application/x-neutrino-sheet';
const SLIDES_MIME = 'application/x-neutrino-slide';

interface ContextMenuState {
  file: FileItem;
  x: number;
  y: number;
}

export default function DrivePage() {
  const queryClient = useQueryClient();
  const toast = useToast();
  const router = useRouter();

  const [sortBy, setSortBy] = useState<SortField>('updatedAt');
  const [sortDir, setSortDir] = useState<SortDir>('desc');
  const [uploadOpen, setUploadOpen] = useState(false);
  const [previewFile, setPreviewFile] = useState<FileItem | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [infoFile, setInfoFile] = useState<FileItem | null>(null);
  const [shareFile, setShareFile] = useState<FileItem | null>(null);
  const [renaming, setRenaming] = useState<FileItem | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const renameInputRef = useRef<HTMLInputElement>(null);
  const [currentFolderId, setCurrentFolderId] = useState<string | null>(null);
  const [folderPath, setFolderPath] = useState<Array<{ id: string; name: string }>>([]);

  useEffect(() => {
    if (renaming && renameInputRef.current) {
      renameInputRef.current.focus();
      renameInputRef.current.select();
    }
  }, [renaming]);

  const { data: starredData } = useQuery({
    queryKey: ['starred'],
    queryFn: () => filesystemApi.getStarred(5),
  });

  const { data: contentsData, isLoading, isError } = useQuery({
    queryKey: ['contents', currentFolderId, { orderBy: sortBy, direction: sortDir }],
    queryFn: () =>
      currentFolderId
        ? filesystemApi.getFolderContents(currentFolderId, { limit: 200, offset: 0, orderBy: sortBy, direction: sortDir })
        : filesystemApi.getRootContents({ limit: 200, offset: 0, orderBy: sortBy, direction: sortDir }),
  });

  const folders: FolderItem[] = contentsData?.folders ?? [];
  const files: FileItem[] = contentsData?.files ?? [];

  // Build lookup maps for context menu callbacks (need original objects)
  const fileMap = new Map(files.map((f) => [f.id, f]));
  const folderMap = new Map(folders.map((f) => [f.id, f]));

  const gridItems: GridItem[] = [
    ...folders.map(folderToGridItem),
    ...files.map(fileToGridItem),
  ];

  function openFolder(folder: FolderItem) {
    setCurrentFolderId(folder.id);
    setFolderPath((prev) => [...prev, { id: folder.id, name: folder.name }]);
  }

  function navigateTo(index: number) {
    if (index === -1) {
      setCurrentFolderId(null);
      setFolderPath([]);
    } else {
      const target = folderPath[index];
      setCurrentFolderId(target.id);
      setFolderPath((prev) => prev.slice(0, index + 1));
    }
  }

  const createDocMutation = useMutation({
    mutationFn: (title: string) => docsApi.createDoc({ title, folderId: currentFolderId }),
    onSuccess: (doc) => {
      queryClient.invalidateQueries({ queryKey: ['contents'] });
      router.push(`/docs/editor?id=${doc.id}`);
    },
    onError: () => toast.error('Failed to create document'),
  });

  const createSheetMutation = useMutation({
    mutationFn: (title: string) => sheetsApi.createSheet({ title, folderId: currentFolderId }),
    onSuccess: (sheet: { id: string }) => {
      queryClient.invalidateQueries({ queryKey: ['contents'] });
      router.push(`/sheets/editor?id=${sheet.id}`);
    },
    onError: () => toast.error('Failed to create spreadsheet'),
  });

  const createSlideMutation = useMutation({
    mutationFn: (title: string) => slidesApi.createSlide({ title, folderId: currentFolderId }),
    onSuccess: (slide: { id: string }) => {
      queryClient.invalidateQueries({ queryKey: ['contents'] });
      router.push(`/slides/editor?id=${slide.id}`);
    },
    onError: () => toast.error('Failed to create presentation'),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, body }: { id: string; body: { name?: string; folderId?: string | null; isStarred?: boolean } }) =>
      filesystemApi.updateFile(id, body),
    onSuccess: (_, { body }) => {
      queryClient.invalidateQueries({ queryKey: ['contents'] });
      if (body.isStarred !== undefined) {
        queryClient.invalidateQueries({ queryKey: ['starred'] });
      }
    },
  });

  const updateFolderMutation = useMutation({
    mutationFn: ({ id, body }: { id: string; body: { name?: string; isStarred?: boolean } }) =>
      filesystemApi.updateFolder(id, body),
    onSuccess: (_, { body }) => {
      queryClient.invalidateQueries({ queryKey: ['contents'] });
      if (body.isStarred !== undefined) {
        queryClient.invalidateQueries({ queryKey: ['starred'] });
      }
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => storageApi.deleteFile(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['contents'] });
      toast.success('File deleted');
    },
    onError: () => toast.error('Failed to delete file'),
  });

  function handleGridItemClick(item: GridItem) {
    if (item.kind === 'folder') {
      const folder = folderMap.get(item.id);
      if (folder) openFolder(folder);
      return;
    }
    const file = fileMap.get(item.id);
    if (!file) return;
    if (file.mimeType === DOC_MIME) {
      router.push(`/docs/editor?id=${file.id}`);
    } else if (file.mimeType === SHEET_MIME) {
      router.push(`/sheets/editor?id=${file.id}`);
    } else if (file.mimeType === SLIDES_MIME) {
      router.push(`/slides/editor?id=${file.id}`);
    } else {
      setPreviewFile(file);
    }
  }

  function handleGridItemMenuOpen(item: GridItem, e: React.MouseEvent) {
    const file = fileMap.get(item.id);
    if (!file) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = Math.min(rect.right, window.innerWidth - 200);
    const y = Math.min(rect.bottom, window.innerHeight - 300);
    setContextMenu({ file, x, y });
  }

  function handleStar(file: FileItem) {
    updateMutation.mutate(
      { id: file.id, body: { isStarred: !file.isStarred } },
      {
        onSuccess: () => toast.success(file.isStarred ? 'Removed from starred' : 'Added to starred'),
        onError: () => toast.error('Failed to update file'),
      }
    );
  }

  function handleToggleStar(item: GridItem) {
    if (item.kind === 'folder') {
      const folder = folderMap.get(item.id);
      if (!folder) return;
      updateFolderMutation.mutate(
        { id: folder.id, body: { isStarred: !folder.isStarred } },
        {
          onSuccess: () => toast.success(folder.isStarred ? 'Removed from starred' : 'Added to starred'),
          onError: () => toast.error('Failed to update folder'),
        }
      );
    } else {
      const file = fileMap.get(item.id);
      if (file) handleStar(file);
    }
  }

  function handleDownload(file: FileItem) {
    const url = storageApi.getFileDownloadUrl(file.id);
    const a = document.createElement('a');
    a.href = url;
    a.download = file.name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  }

  function handleCopyLink(file: FileItem) {
    const url = storageApi.getFileDownloadUrl(file.id);
    navigator.clipboard.writeText(url).then(
      () => toast.success('Link copied to clipboard'),
      () => toast.error('Failed to copy link')
    );
  }

  function openRename(file: FileItem) {
    setRenameValue(file.name);
    setRenaming(file);
  }

  function handleRenameSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!renaming) return;
    const trimmed = renameValue.trim();
    if (!trimmed || trimmed === renaming.name) { setRenaming(null); return; }
    updateMutation.mutate(
      { id: renaming.id, body: { name: trimmed } },
      {
        onSuccess: () => { toast.success('File renamed'); setRenaming(null); },
        onError: () => toast.error('Failed to rename file'),
      }
    );
  }

  return (
    <div className={styles.page}>
      {/* Page header */}
      <div className={styles.header}>
        <div className={styles['header-left']}>
          <Breadcrumbs
            items={[
              { label: 'My Drive', onClick: folderPath.length > 0 ? () => navigateTo(-1) : undefined },
              ...folderPath.map((f, i) => ({
                label: f.name,
                onClick: i < folderPath.length - 1 ? () => navigateTo(i) : undefined,
              })),
            ]}
          />
          <Heading level={1} size="xl">
            {folderPath.length > 0 ? folderPath[folderPath.length - 1].name : 'My Drive'}
          </Heading>
        </div>
        <div className={styles['header-actions']}>
          <Button variant="ghost" size="sm" icon={<FolderPlus size={16} />}>New folder</Button>
          <Button
            variant="ghost"
            size="sm"
            icon={<FileText size={16} />}
            onClick={() => createDocMutation.mutate('Untitled document')}
            disabled={createDocMutation.isPending}
          >
            New document
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => createSheetMutation.mutate('Untitled spreadsheet')}
            disabled={createSheetMutation.isPending}
          >
            New spreadsheet
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => createSlideMutation.mutate('Untitled presentation')}
            disabled={createSlideMutation.isPending}
          >
            New presentation
          </Button>
          <Button variant="primary" size="sm" icon={<Upload size={16} />} onClick={() => setUploadOpen(true)}>
            Upload
          </Button>
        </div>
      </div>

      {/* Quick access */}
      <section className={styles.section} aria-labelledby="quick-access-heading">
        <div className={styles['section-header']}>
          <Heading level={2} size="sm" id="quick-access-heading">Quick access</Heading>
          <Text as="span" size="xs" color="muted">
            <Clock size={12} style={{ display: 'inline', marginRight: '4px', verticalAlign: 'middle' }} />
            Recently starred
          </Text>
        </div>
        <div className={styles['quick-grid']}>
          {!starredData
            ? Array.from({ length: 4 }, (_, i) => (
                <div key={i} className={styles['quick-card-skeleton']}>
                  <Skeleton shape="rect" width={32} height={32} />
                  <div style={{ flex: 1 }}>
                    <Skeleton shape="text" width="80%" height="0.875rem" />
                    <div style={{ marginTop: '4px' }}><Skeleton shape="text" width="50%" height="0.75rem" /></div>
                  </div>
                </div>
              ))
            : (() => {
                const starredFiles = starredData.files.map((file) => ({
                  key: file.id,
                  icon: getFileIcon(file.mimeType),
                  iconColor: getIconColor(file.mimeType),
                  name: file.name,
                  date: file.updatedAt,
                  onClick: () => {
                    if (file.mimeType === DOC_MIME) router.push(`/docs/editor?id=${file.id}`);
                    else if (file.mimeType === SHEET_MIME) router.push(`/sheets/editor?id=${file.id}`);
                    else if (file.mimeType === SLIDES_MIME) router.push(`/slides/editor?id=${file.id}`);
                    else setPreviewFile(file);
                  },
                }));
                const starredFolders = starredData.folders.map((folder) => ({
                  key: folder.id,
                  icon: Folder,
                  iconColor: folder.color ?? 'var(--color-amber, #d97706)',
                  name: folder.name,
                  date: folder.updatedAt,
                  onClick: () => openFolder(folder),
                }));
                const items = [...starredFiles, ...starredFolders];
                if (items.length === 0) {
                  return (
                    <Text size="sm" color="muted">
                      Star files and folders to see them here.
                    </Text>
                  );
                }
                return items.map((item) => {
                  const IconComponent = item.icon;
                  return (
                    <Card key={item.key} hoverable padding="sm" className={styles['quick-card']} role="button" tabIndex={0} aria-label={`Open ${item.name}`} onClick={item.onClick}>
                      <div className={styles['quick-card-inner']}>
                        <div className={styles['file-icon-sm']} style={{ color: item.iconColor }}>
                          <IconComponent size={20} strokeWidth={1.5} />
                        </div>
                        <div className={styles['quick-card-info']}>
                          <Text size="sm" weight="medium" truncate>{item.name}</Text>
                          <Text size="xs" color="muted">{formatDate(item.date)}</Text>
                        </div>
                      </div>
                    </Card>
                  );
                });
              })()}
        </div>
      </section>

      {/* All files */}
      <section className={styles.section} aria-labelledby="all-files-heading">
        <Heading level={2} size="sm" id="all-files-heading">Files</Heading>
        <FileGrid
          items={gridItems}
          isLoading={isLoading}
          isError={isError}
          emptyState={
            isError ? (
              <EmptyState
                title="Could not load files"
                description="There was an error loading your files. Please try again."
                action={<Button variant="secondary" size="sm" onClick={() => window.location.reload()}>Retry</Button>}
              />
            ) : (
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
            )
          }
          onItemClick={handleGridItemClick}
          onItemMenuOpen={handleGridItemMenuOpen}
          onToggleStar={handleToggleStar}
          showFilter
          showSizeColumn
          sortBy={sortBy}
          sortDir={sortDir}
          onSortChange={(field, dir) => { setSortBy(field); setSortDir(dir); }}
          totalCount={isLoading ? undefined : folders.length + files.length}
        />
      </section>

      {/* Overlays */}
      {uploadOpen && <UploadZone onClose={() => setUploadOpen(false)} folderId={currentFolderId} />}
      {previewFile && <PreviewModal file={previewFile} onClose={() => setPreviewFile(null)} />}

      {contextMenu && (
        <FileContextMenu
          file={contextMenu.file}
          x={contextMenu.x}
          y={contextMenu.y}
          onClose={() => setContextMenu(null)}
          onInfo={() => { setInfoFile(contextMenu.file); setContextMenu(null); }}
          onShare={() => { setShareFile(contextMenu.file); setContextMenu(null); }}
          onRename={() => { openRename(contextMenu.file); setContextMenu(null); }}
          onStarToggle={() => { handleStar(contextMenu.file); setContextMenu(null); }}
          onDownload={() => { handleDownload(contextMenu.file); setContextMenu(null); }}
          onDelete={() => { deleteMutation.mutate(contextMenu.file.id); setContextMenu(null); }}
          onCopyLink={() => { handleCopyLink(contextMenu.file); setContextMenu(null); }}
        />
      )}

      {shareFile && (
        <ShareDialog resource={shareFile} resourceType="file" onClose={() => setShareFile(null)} />
      )}

      {infoFile && <FileInfoPanel file={infoFile} onClose={() => setInfoFile(null)} />}

      {renaming && (
        <div className={styles['rename-overlay']} onClick={() => setRenaming(null)}>
          <div className={styles['rename-dialog']} onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true" aria-labelledby="rename-title">
            <Heading level={2} size="sm" id="rename-title">Rename file</Heading>
            <form className={styles['rename-form']} onSubmit={handleRenameSubmit}>
              <input
                ref={renameInputRef}
                className={styles['rename-input']}
                type="text"
                value={renameValue}
                onChange={(e) => setRenameValue(e.target.value)}
                aria-label="New file name"
              />
              <div className={styles['rename-actions']}>
                <Button type="button" variant="ghost" size="sm" onClick={() => setRenaming(null)}>Cancel</Button>
                <Button type="submit" variant="primary" size="sm" disabled={!renameValue.trim()}>Rename</Button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
