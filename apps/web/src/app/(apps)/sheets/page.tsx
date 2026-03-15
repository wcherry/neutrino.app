'use client';

import React from 'react';
import { useRouter } from 'next/navigation';
import { useMutation, useQuery } from '@tanstack/react-query';
import { Button, EmptyState, Heading } from '@neutrino/ui';
import { FilePlus, Table2 } from 'lucide-react';
import { sheetsApi, type SheetMetaResponse } from '@/lib/api';
import { FileGrid, type GridItem, type SortField, type SortDir } from '@/components/FileGrid';
import styles from './page.module.css';

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

function sheetToGridItem(sheet: SheetMetaResponse): GridItem {
  return {
    id: sheet.id,
    name: sheet.title,
    kind: 'doc',
    icon: Table2,
    iconColor: 'var(--color-green, #16a34a)',
    subtitle: formatDate(sheet.updatedAt),
    typeText: 'Sheet',
    modifiedText: formatDate(sheet.updatedAt),
  };
}

export default function SheetsPage() {
  const router = useRouter();
  const [sortBy, setSortBy] = React.useState<SortField>('updatedAt');
  const [sortDir, setSortDir] = React.useState<SortDir>('desc');

  const { data, isLoading, isError } = useQuery({
    queryKey: ['sheets'],
    queryFn: () => sheetsApi.listSheets(),
  });

  const createSheet = useMutation({
    mutationFn: () => sheetsApi.createSheet({ title: 'Untitled spreadsheet' }),
    onSuccess: (sheet) => router.push(`/sheets/editor?id=${sheet.id}`),
  });

  const sheets = data?.sheets ?? [];
  const gridItems: GridItem[] = sheets.map(sheetToGridItem);

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <Heading level={1} size="xl">Spreadsheets</Heading>
        <Button onClick={() => createSheet.mutate()} disabled={createSheet.isPending} icon={<FilePlus size={16} />}>
          New Spreadsheet
        </Button>
      </div>

      <FileGrid
        items={gridItems}
        isLoading={isLoading}
        isError={isError}
        emptyState={
          <EmptyState
            icon={FilePlus}
            title="No spreadsheets yet"
            description="Create a new spreadsheet to get started."
            action={
              <Button onClick={() => createSheet.mutate()} disabled={createSheet.isPending}>
                New Spreadsheet
              </Button>
            }
          />
        }
        onItemClick={(item) => router.push(`/sheets/editor?id=${item.id}`)}
        showFilter={false}
        showSizeColumn={false}
        sortBy={sortBy}
        sortDir={sortDir}
        onSortChange={(field, dir) => { setSortBy(field); setSortDir(dir); }}
        totalCount={isLoading ? undefined : sheets.length}
      />
    </div>
  );
}
