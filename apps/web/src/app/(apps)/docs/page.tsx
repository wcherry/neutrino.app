'use client';

import React from 'react';
import { useRouter } from 'next/navigation';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Button, EmptyState, Heading } from '@neutrino/ui';
import { FilePlus, FileText } from 'lucide-react';
import { docsApi, type DocMetaResponse } from '@/lib/api';
import { FileGrid, type GridItem, type SortField, type SortDir } from '@/components/FileGrid';
import styles from './page.module.css';

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function docToGridItem(doc: DocMetaResponse): GridItem {
  return {
    id: doc.id,
    name: doc.title,
    kind: 'doc',
    icon: FileText,
    iconColor: 'var(--color-accent)',
    subtitle: formatDate(doc.updatedAt),
    typeText: 'Doc',
    modifiedText: formatDate(doc.updatedAt),
  };
}

export default function DocsPage() {
  const router = useRouter();
  const [sortBy, setSortBy] = React.useState<SortField>('updatedAt');
  const [sortDir, setSortDir] = React.useState<SortDir>('desc');

  const { data, isLoading, isError } = useQuery({
    queryKey: ['docs'],
    queryFn: () => docsApi.listDocs(),
  });

  const createDoc = useMutation({
    mutationFn: () => docsApi.createDoc({ title: 'Untitled document' }),
    onSuccess: (doc) => router.push(`/docs/editor?id=${doc.id}`),
  });

  const docs = data?.docs ?? [];
  const gridItems: GridItem[] = docs.map(docToGridItem);

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <Heading level={1} size="xl">Documents</Heading>
        <Button onClick={() => createDoc.mutate()} disabled={createDoc.isPending} icon={<FilePlus size={16} />}>
          New Document
        </Button>
      </div>

      <FileGrid
        items={gridItems}
        isLoading={isLoading}
        isError={isError}
        emptyState={
          <EmptyState
            icon={FilePlus}
            title="No documents yet"
            description="Create a new document to get started."
            action={
              <Button onClick={() => createDoc.mutate()} disabled={createDoc.isPending}>
                New Document
              </Button>
            }
          />
        }
        onItemClick={(item) => router.push(`/docs/editor?id=${item.id}`)}
        showFilter={false}
        showSizeColumn={false}
        sortBy={sortBy}
        sortDir={sortDir}
        onSortChange={(field, dir) => { setSortBy(field); setSortDir(dir); }}
        totalCount={isLoading ? undefined : docs.length}
      />
    </div>
  );
}
