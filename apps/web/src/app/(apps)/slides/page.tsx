'use client';

import React, { useRef, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Button, EmptyState, Heading } from '@neutrino/ui';
import { FilePlus, Presentation, Upload } from 'lucide-react';
import { slidesApi, type SlideMetaResponse } from '@/lib/api';
import { FileGrid, type GridItem, type SortField, type SortDir } from '@/components/FileGrid';
import { importFromPptx } from './editor/SlideEditor';
import styles from './page.module.css';

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function slideToGridItem(slide: SlideMetaResponse): GridItem {
  return {
    id: slide.id,
    name: slide.title,
    kind: 'doc',
    icon: Presentation,
    iconColor: 'var(--color-rose, #e11d48)',
    subtitle: formatDate(slide.updatedAt),
    typeText: 'Presentation',
    modifiedText: formatDate(slide.updatedAt),
  };
}

export default function SlidesPage() {
  const router = useRouter();
  const [sortBy, setSortBy] = React.useState<SortField>('updatedAt');
  const [sortDir, setSortDir] = React.useState<SortDir>('desc');
  const [importError, setImportError] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const importInputRef = useRef<HTMLInputElement>(null);

  const { data, isLoading, isError } = useQuery({
    queryKey: ['slides'],
    queryFn: () => slidesApi.listSlides(),
  });

  const createSlide = useMutation({
    mutationFn: () => slidesApi.createSlide({ title: 'Untitled presentation' }),
    onSuccess: (slide) => router.push(`/slides/editor?id=${slide.id}`),
  });

  const createAndImport = useMutation({
    mutationFn: async (file: File) => {
      const fileName = file.name.replace(/\.pptx$/i, '') || 'Imported presentation';
      const slide = await slidesApi.createSlide({ title: fileName });
      const presentation = await importFromPptx(file);
      await slidesApi.saveSlide(slide.id, { title: fileName });
      return slide;
    },
    onSuccess: (slide) => router.push(`/slides/editor?id=${slide.id}`),
    onError: (err) => setImportError(err instanceof Error ? err.message : 'Import failed'),
    onSettled: () => setImporting(false),
  });

  async function handleImportFile(file: File) {
    setImportError(null);
    setImporting(true);
    createAndImport.mutate(file);
  }

  const slides = data?.slides ?? [];
  const gridItems: GridItem[] = slides.map(slideToGridItem);

  return (
    <div className={styles.page}>
      {/* Hidden file input */}
      <input
        ref={importInputRef}
        type="file"
        accept=".pptx,application/vnd.openxmlformats-officedocument.presentationml.presentation"
        style={{ display: 'none' }}
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) handleImportFile(file);
          e.target.value = '';
        }}
      />

      <div className={styles.header}>
        <Heading level={1} size="xl">Presentations</Heading>
        <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
          <Button
            variant="secondary"
            onClick={() => importInputRef.current?.click()}
            disabled={importing}
            icon={<Upload size={16} />}
          >
            {importing ? 'Importing…' : 'Import PPTX'}
          </Button>
          <Button onClick={() => createSlide.mutate()} disabled={createSlide.isPending} icon={<FilePlus size={16} />}>
            New Presentation
          </Button>
        </div>
      </div>

      {importError && (
        <div style={{ padding: 'var(--space-3) var(--space-4)', background: 'var(--color-danger-subtle, #fef2f2)', border: '1px solid var(--color-danger, #dc2626)', borderRadius: 'var(--radius-md)', color: 'var(--color-danger, #dc2626)', fontSize: 'var(--font-size-sm)', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          {importError}
          <button onClick={() => setImportError(null)} style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'inherit' }}>✕</button>
        </div>
      )}

      <FileGrid
        items={gridItems}
        isLoading={isLoading}
        isError={isError}
        emptyState={
          <EmptyState
            icon={FilePlus}
            title="No presentations yet"
            description="Create a new presentation to get started."
            action={
              <Button onClick={() => createSlide.mutate()} disabled={createSlide.isPending}>
                New Presentation
              </Button>
            }
          />
        }
        onItemClick={(item) => router.push(`/slides/editor?id=${item.id}`)}
        showFilter={false}
        showSizeColumn={false}
        sortBy={sortBy}
        sortDir={sortDir}
        onSortChange={(field, dir) => { setSortBy(field); setSortDir(dir); }}
        totalCount={isLoading ? undefined : slides.length}
      />
    </div>
  );
}
