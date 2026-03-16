'use client';

import React, { useState, useCallback, useRef, useEffect } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, Download, Upload, ChevronDown, Table2 } from 'lucide-react';
import { Button } from '@neutrino/ui';
import { sheetsApi, driveReadContent, driveWriteContent } from '@/lib/api';
import styles from './page.module.css';

// ── Export helpers ──────────────────────────────────────────────────────────

async function exportAsCsv(title: string, data: object[]) {
  const XLSX = await import('xlsx');
  const ws = XLSX.utils.json_to_sheet(data);
  const csv = XLSX.utils.sheet_to_csv(ws);
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${title}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

async function exportAsXlsx(title: string, fortuneData: object[]) {
  const XLSX = await import('xlsx');
  const wb = XLSX.utils.book_new();
  // Convert FortuneSheet celldata to aoa for each sheet
  for (const sheet of fortuneData as Array<{
    name: string;
    celldata: Array<{ r: number; c: number; v: { v?: string | number | boolean; ct?: { t: string }; m?: string } }>;
    row: number;
    column: number;
  }>) {
    const rows: (string | number | boolean | null)[][] = [];
    for (const cell of sheet.celldata) {
      while (rows.length <= cell.r) rows.push([]);
      const row = rows[cell.r];
      while (row.length <= cell.c) row.push(null);
      row[cell.c] = cell.v?.m ?? cell.v?.v ?? null;
    }
    const ws = XLSX.utils.aoa_to_sheet(rows);
    XLSX.utils.book_append_sheet(wb, ws, sheet.name || 'Sheet1');
  }
  XLSX.writeFile(wb, `${title}.xlsx`);
}

async function importFromFile(file: File): Promise<object[]> {
  const XLSX = await import('xlsx');
  const buffer = await file.arrayBuffer();
  const wb = XLSX.read(buffer, { type: 'array' });

  return wb.SheetNames.map((name, idx) => {
    const ws = wb.Sheets[name];
    const aoa: (string | number | boolean | null)[][] = XLSX.utils.sheet_to_json(ws, { header: 1, defval: null });
    const celldata: Array<{ r: number; c: number; v: { v: string | number | boolean | null; ct: { t: string }; m: string } }> = [];
    for (let r = 0; r < aoa.length; r++) {
      for (let c = 0; c < (aoa[r]?.length ?? 0); c++) {
        const val = aoa[r][c];
        if (val !== null && val !== undefined && val !== '') {
          celldata.push({
            r,
            c,
            v: {
              v: val,
              ct: { t: typeof val === 'number' ? 'n' : 'g' },
              m: String(val),
            },
          });
        }
      }
    }
    return {
      index: String(idx),
      name,
      celldata,
      row: Math.max(100, aoa.length + 10),
      column: 26,
      order: idx,
      status: idx === 0 ? 1 : 0,
      config: {},
    };
  });
}

// ── Save status types ───────────────────────────────────────────────────────

type SaveStatus = 'saved' | 'saving' | 'unsaved' | 'error';

// ── Main component ──────────────────────────────────────────────────────────

export function SheetEditor() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const queryClient = useQueryClient();
  const sheetId = searchParams.get('id') ?? '';

  const [title, setTitle] = useState('');
  const [saveStatus, setSaveStatus] = useState<SaveStatus>('saved');
  const [exportOpen, setExportOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const importInputRef = useRef<HTMLInputElement>(null);
  const exportRef = useRef<HTMLDivElement>(null);
  const importRef = useRef<HTMLDivElement>(null);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastSavedContentRef = useRef<string>('');
  // Holds the latest FortuneSheet data for saving WITHOUT feeding back into the
  // `data` prop (which would cause FortuneSheet to reinitialize and break formulas).
  const latestDataRef = useRef<object[]>([]);

  // FortuneSheet data state (array of sheet objects) — only used as the INITIAL
  // value passed to the Workbook. After mount the Workbook owns the state.
  const [fortuneData, setFortuneData] = useState<object[]>([
    { index: '0', name: 'Sheet1', celldata: [], row: 100, column: 26, order: 0, status: 1, config: {} },
  ]);

  // Whether FortuneSheet component is mounted (client-only)
  const [mounted, setMounted] = useState(false);
  useEffect(() => { setMounted(true); }, []);

  const { isLoading: metaLoading, data: sheetData } = useQuery({
    queryKey: ['sheet', sheetId],
    queryFn: () => sheetsApi.getSheet(sheetId),
    enabled: !!sheetId,
    staleTime: 30_000,
  });

  const { isLoading: contentLoading, data: sheetContent } = useQuery({
    queryKey: ['sheet-content', sheetId],
    queryFn: () => driveReadContent(sheetData!.contentUrl),
    enabled: !!sheetData?.contentUrl,
    staleTime: 30_000,
  });

  const isLoading = metaLoading || contentLoading;

  useEffect(() => {
    if (!sheetData) return;
    setTitle(sheetData.title);
  }, [sheetData]);

  useEffect(() => {
    if (!sheetContent) return;
    try {
      const parsed = JSON.parse(sheetContent);
      if (Array.isArray(parsed) && parsed.length > 0) {
        setFortuneData(parsed);
        lastSavedContentRef.current = sheetContent;
      }
    } catch {
      // keep default
    }
  }, [sheetContent]);

  const contentMutation = useMutation({
    mutationFn: (content: string) =>
      driveWriteContent(sheetData!.contentWriteUrl, content, 'sheet.json'),
    onMutate: () => setSaveStatus('saving'),
    onSuccess: (_, content) => {
      setSaveStatus('saved');
      lastSavedContentRef.current = content;
      queryClient.invalidateQueries({ queryKey: ['sheets'] });
    },
    onError: () => setSaveStatus('error'),
  });

  const metaMutation = useMutation({
    mutationFn: (newTitle: string) =>
      sheetsApi.saveSheet(sheetId, { title: newTitle }),
  });

  const scheduleAutoSave = useCallback(() => {
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    setSaveStatus('unsaved');
    saveTimerRef.current = setTimeout(() => {
      const content = JSON.stringify(latestDataRef.current);
      contentMutation.mutate(content);
    }, 15000);
  }, [contentMutation]);

  function handleChange(data: object[]) {
    const serialized = JSON.stringify(data);
    if (serialized === lastSavedContentRef.current) return;
    // Store latest data in ref only — do NOT call setFortuneData here.
    // Feeding onChange output back into the `data` prop causes FortuneSheet to
    // reinitialize its context on every keystroke, which breaks formula evaluation.
    latestDataRef.current = data;
    scheduleAutoSave();
  }

  function handleTitleBlur() {
    const trimmed = title.trim();
    if (!trimmed) return;
    metaMutation.mutate(trimmed);
  }

  async function handleExportCsv() {
    setExportOpen(false);
    await exportAsCsv(title || 'spreadsheet', latestDataRef.current.length ? latestDataRef.current : fortuneData);
  }

  async function handleExportXlsx() {
    setExportOpen(false);
    await exportAsXlsx(title || 'spreadsheet', latestDataRef.current.length ? latestDataRef.current : fortuneData);
  }

  async function handleImport(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    try {
      const imported = await importFromFile(file);
      latestDataRef.current = imported;
      setFortuneData(imported);
      scheduleAutoSave();
    } catch (err) {
      console.error('Import failed:', err);
    } finally {
      if (importInputRef.current) importInputRef.current.value = '';
    }
  }

  // Close dropdowns on outside click
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (exportRef.current && !exportRef.current.contains(e.target as Node)) {
        setExportOpen(false);
      }
      if (importRef.current && !importRef.current.contains(e.target as Node)) {
        setImportOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  // Cleanup save timer on unmount
  useEffect(() => {
    return () => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    };
  }, []);

  const saveStatusText =
    saveStatus === 'saving' ? 'Saving…' :
    saveStatus === 'unsaved' ? 'Unsaved changes' :
    saveStatus === 'error' ? 'Save failed' :
    'All changes saved';

  const saveStatusClass =
    saveStatus === 'saving' ? styles.saveStatusSaving :
    saveStatus === 'error' ? styles.saveStatusError :
    '';

  if (isLoading) {
    return <div style={{ padding: '2rem' }}>Loading spreadsheet…</div>;
  }

  return (
    <div className={styles.editorWrapper}>
      {/* Top bar */}
      <div className={styles.topBar}>
        <Button
          variant="ghost"
          icon={<ArrowLeft size={16} />}
          onClick={() => router.push('/sheets')}
          className={styles.backBtn}
        >
          Sheets
        </Button>

        <div className={styles.titleArea}>
          <Table2 size={18} color="var(--color-green, #16a34a)" />
          <input
            className={styles.titleInput}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            onBlur={handleTitleBlur}
            onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur(); }}
            placeholder="Untitled spreadsheet"
          />
        </div>

        <span className={`${styles.saveStatus} ${saveStatusClass}`}>
          {saveStatusText}
        </span>

        <div className={styles.actions}>
          {/* Import */}
          <div className={styles.dropdownTrigger} ref={importRef}>
            <Button
              variant="secondary"
              icon={<Upload size={16} />}
              onClick={() => setImportOpen((v) => !v)}
            >
              Import
            </Button>
            {importOpen && (
              <div className={styles.dropdownMenu}>
                <button
                  className={styles.dropdownItem}
                  onClick={() => { setImportOpen(false); importInputRef.current?.click(); }}
                >
                  CSV / TSV file
                </button>
                <button
                  className={styles.dropdownItem}
                  onClick={() => { setImportOpen(false); importInputRef.current?.click(); }}
                >
                  Excel (.xlsx)
                </button>
              </div>
            )}
          </div>
          <input
            ref={importInputRef}
            type="file"
            accept=".csv,.tsv,.xlsx,.xls"
            style={{ display: 'none' }}
            onChange={handleImport}
          />

          {/* Export */}
          <div className={styles.dropdownTrigger} ref={exportRef}>
            <Button
              variant="secondary"
              icon={<Download size={16} />}
              onClick={() => setExportOpen((v) => !v)}
            >
              Export <ChevronDown size={14} />
            </Button>
            {exportOpen && (
              <div className={styles.dropdownMenu}>
                <button className={styles.dropdownItem} onClick={handleExportXlsx}>
                  Excel (.xlsx)
                </button>
                <button className={styles.dropdownItem} onClick={handleExportCsv}>
                  CSV (.csv)
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Spreadsheet grid */}
      <div className={styles.gridArea}>
        {mounted && (
          <FortuneSheetWorkbook data={fortuneData} onChange={handleChange} />
        )}
      </div>
    </div>
  );
}

// ── FortuneSheet wrapper (dynamic import to avoid SSR issues) ───────────────

// FortuneSheet v0.4.2 bug: the "image" toolbar item returns a React.Fragment
// without a key inside .map(), triggering a React key warning. Exclude it.
const TOOLBAR_ITEMS = [
  'undo', 'redo', 'format-painter', 'clear-format', '|',
  'currency-format', 'percentage-format', 'number-decrease', 'number-increase', 'format', 'font-size', '|',
  'bold', 'italic', 'strike-through', 'underline', '|',
  'font-color', 'background', 'border', 'merge-cell', '|',
  'align-left', 'align-center', 'align-right', 'align-top', 'align-mid', 'align-bottom', '|',
  'freeze', 'comment', 'formula-sum',
];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type WorkbookComponent = React.ComponentType<any>;

function FortuneSheetWorkbook({
  data,
  onChange,
}: {
  data: object[];
  onChange: (data: object[]) => void;
}) {
  const [Workbook, setWorkbook] = useState<WorkbookComponent | null>(null);

  useEffect(() => {
    import('@fortune-sheet/react').then((mod) => {
      // Import the CSS
      // @ts-expect-error — no type declarations for CSS module
      import('@fortune-sheet/react/dist/index.css').catch(() => {});
      setWorkbook(() => mod.Workbook as WorkbookComponent);
    });
  }, []);

  if (!Workbook) {
    return <div style={{ padding: '2rem', color: 'var(--color-text-muted)' }}>Loading spreadsheet engine…</div>;
  }

  return (
    <Workbook
      data={data}
      onChange={onChange}
      allowEdit
      showToolbar
      showFormulaBar
      showSheetTabs
      toolbarItems={TOOLBAR_ITEMS}
      style={{ height: '100%', width: '100%' }}
    />
  );
}
