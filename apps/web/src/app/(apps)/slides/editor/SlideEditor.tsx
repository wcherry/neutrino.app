'use client';

import React, {
  useState,
  useCallback,
  useRef,
  useEffect,
} from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  ArrowLeft,
  Download,
  Plus,
  Trash2,
  Type,
  Square,
  Circle,
  ChevronDown,
  Play,
  Presentation,
  Copy,
  Bold,
  Italic,
  Underline,
  AlignLeft,
  AlignCenter,
  AlignRight,
  Minus,
  ChevronUp,
  LayoutTemplate,
  Zap,
  Upload,
} from 'lucide-react';
import { Button } from '@neutrino/ui';
import { slidesApi, driveReadContent, driveWriteContent } from '@/lib/api';
import styles from './page.module.css';

// ── Data model ──────────────────────────────────────────────────────────────

export interface TextStyle {
  fontSize: number;
  bold: boolean;
  italic: boolean;
  underline: boolean;
  color: string;
  align: 'left' | 'center' | 'right';
  fontFamily: string;
}

export interface ElementAnimation {
  type: 'none' | 'fade' | 'fly-in' | 'zoom';
  duration: number; // ms
  delay: number; // ms
  direction?: 'left' | 'right' | 'top' | 'bottom';
}

export interface TextElement {
  id: string;
  type: 'text';
  x: number; // percentage 0-100
  y: number;
  w: number;
  h: number;
  content: string;
  style: TextStyle;
  animation?: ElementAnimation;
}

export interface ShapeElement {
  id: string;
  type: 'shape';
  shape: 'rect' | 'circle';
  x: number;
  y: number;
  w: number;
  h: number;
  fill: string;
  stroke: string;
  strokeWidth: number;
  animation?: ElementAnimation;
}

export type SlideElement = TextElement | ShapeElement;

export interface Slide {
  id: string;
  background: { type: 'color'; value: string };
  elements: SlideElement[];
  notes: string;
  transition: 'none' | 'fade' | 'slide' | 'zoom';
}

export interface Theme {
  name: string;
  primaryColor: string;
  backgroundColor: string;
  textColor: string;
  accentColor: string;
}

export interface SlideMaster {
  background: string;
  titleFontSize: number;
  titleBold: boolean;
  titleColor: string;
  bodyFontSize: number;
  bodyBold: boolean;
  bodyColor: string;
}

export interface SlidePresentation {
  slides: Slide[];
  theme: Theme;
  master?: SlideMaster;
}

// ── Built-in themes ─────────────────────────────────────────────────────────

const THEMES: Theme[] = [
  { name: 'Default', primaryColor: '#4f46e5', backgroundColor: '#ffffff', textColor: '#1f2937', accentColor: '#818cf8' },
  { name: 'Dark', primaryColor: '#6366f1', backgroundColor: '#111827', textColor: '#f9fafb', accentColor: '#a5b4fc' },
  { name: 'Ocean', primaryColor: '#0284c7', backgroundColor: '#f0f9ff', textColor: '#0c4a6e', accentColor: '#38bdf8' },
  { name: 'Forest', primaryColor: '#15803d', backgroundColor: '#f0fdf4', textColor: '#14532d', accentColor: '#4ade80' },
  { name: 'Sunset', primaryColor: '#ea580c', backgroundColor: '#fff7ed', textColor: '#7c2d12', accentColor: '#fb923c' },
  { name: 'Minimal', primaryColor: '#374151', backgroundColor: '#f9fafb', textColor: '#111827', accentColor: '#9ca3af' },
];

// ── Default presentation and master ─────────────────────────────────────────

function makeDefaultPresentation(): SlidePresentation {
  return {
    slides: [
      {
        id: uid(),
        background: { type: 'color', value: '#ffffff' },
        elements: [
          {
            id: uid(),
            type: 'text',
            x: 10, y: 30, w: 80, h: 20,
            content: 'Click to add title',
            style: { fontSize: 40, bold: true, italic: false, underline: false, color: '#1f2937', align: 'center', fontFamily: 'Inter' },
          },
          {
            id: uid(),
            type: 'text',
            x: 15, y: 55, w: 70, h: 15,
            content: 'Click to add subtitle',
            style: { fontSize: 24, bold: false, italic: false, underline: false, color: '#6b7280', align: 'center', fontFamily: 'Inter' },
          },
        ],
        notes: '',
        transition: 'fade',
      },
    ],
    theme: THEMES[0],
    master: makeDefaultMaster(),
  };
}

function makeDefaultMaster(): SlideMaster {
  return {
    background: '#ffffff',
    titleFontSize: 40,
    titleBold: true,
    titleColor: '#1f2937',
    bodyFontSize: 24,
    bodyBold: false,
    bodyColor: '#6b7280',
  };
}

function uid() {
  return Math.random().toString(36).slice(2, 10);
}

// ── PPTX export ──────────────────────────────────────────────────────────────

async function exportAsPptx(title: string, presentation: SlidePresentation) {
  const pptxgen = (await import('pptxgenjs')).default;
  const prs = new pptxgen();
  prs.layout = 'LAYOUT_16x9';

  for (const slide of presentation.slides) {
    const pSlide = prs.addSlide();
    pSlide.background = { color: slide.background.value.replace('#', '') };

    for (const el of slide.elements) {
      if (el.type === 'text') {
        pSlide.addText(el.content, {
          x: `${el.x}%`,
          y: `${el.y}%`,
          w: `${el.w}%`,
          h: `${el.h}%`,
          fontSize: el.style.fontSize * 0.75, // pt conversion
          bold: el.style.bold,
          italic: el.style.italic,
          underline: el.style.underline ? { style: 'sng' } : undefined,
          color: el.style.color.replace('#', ''),
          align: el.style.align,
          fontFace: el.style.fontFamily,
          wrap: true,
        });
      } else if (el.type === 'shape') {
        pSlide.addShape(prs.ShapeType.rect, {
          x: `${el.x}%`,
          y: `${el.y}%`,
          w: `${el.w}%`,
          h: `${el.h}%`,
          fill: { color: el.fill.replace('#', '') },
          line: { color: el.stroke.replace('#', ''), width: el.strokeWidth },
        });
      }
    }

    if (slide.notes) {
      pSlide.addNotes(slide.notes);
    }
  }

  await prs.writeFile({ fileName: `${title}.pptx` });
}

// ── PPTX import ──────────────────────────────────────────────────────────────

const PPTX_MAX_BYTES = 100 * 1024 * 1024; // 100 MB
const NS_A = 'http://schemas.openxmlformats.org/drawingml/2006/main';
const NS_P = 'http://schemas.openxmlformats.org/presentationml/2006/main';
// Standard 16:9 slide dimensions in EMU
const SLIDE_W_EMU = 9144000;
const SLIDE_H_EMU = 5143500;

export async function importFromPptx(file: File): Promise<SlidePresentation> {
  if (file.size > PPTX_MAX_BYTES) {
    throw new Error('File size exceeds 100 MB limit');
  }

  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore – jszip is installed at runtime; types resolve after pnpm install
  const JSZip = (await import('jszip')).default;
  const zip = await JSZip.loadAsync(file);

  // Find and sort slide files
  const slideFiles = Object.keys(zip.files)
    .filter(name => /^ppt\/slides\/slide\d+\.xml$/.test(name))
    .sort((a, b) => {
      const numA = parseInt(a.match(/(\d+)\.xml$/)![1]);
      const numB = parseInt(b.match(/(\d+)\.xml$/)![1]);
      return numA - numB;
    });

  if (slideFiles.length === 0) {
    return makeDefaultPresentation();
  }

  const parser = new DOMParser();
  const slides: Slide[] = [];

  for (const slidePath of slideFiles) {
    const xml = await zip.files[slidePath].async('text');
    const doc = parser.parseFromString(xml, 'application/xml');

    // Background color
    let bgColor = '#ffffff';
    const bgNode = doc.getElementsByTagNameNS(NS_P, 'bg')[0];
    if (bgNode) {
      const bgSolid = bgNode.getElementsByTagNameNS(NS_A, 'solidFill')[0];
      if (bgSolid) {
        const srgb = bgSolid.getElementsByTagNameNS(NS_A, 'srgbClr')[0];
        if (srgb) bgColor = `#${srgb.getAttribute('val') ?? 'ffffff'}`;
      }
    }

    // Parse shapes from spTree
    const spTree = doc.getElementsByTagNameNS(NS_P, 'spTree')[0];
    const elements: SlideElement[] = [];

    if (spTree) {
      const spNodes = Array.from(spTree.getElementsByTagNameNS(NS_P, 'sp'));
      for (const sp of spNodes) {
        const spPr = sp.getElementsByTagNameNS(NS_P, 'spPr')[0];
        const xfrm = spPr?.getElementsByTagNameNS(NS_A, 'xfrm')[0];
        if (!xfrm) continue;

        const off = xfrm.getElementsByTagNameNS(NS_A, 'off')[0];
        const ext = xfrm.getElementsByTagNameNS(NS_A, 'ext')[0];
        if (!off || !ext) continue;

        const x = parseInt(off.getAttribute('x') ?? '0');
        const y = parseInt(off.getAttribute('y') ?? '0');
        const cx = parseInt(ext.getAttribute('cx') ?? '0');
        const cy = parseInt(ext.getAttribute('cy') ?? '0');

        const txBody = sp.getElementsByTagNameNS(NS_P, 'txBody')[0];
        if (!txBody) continue;

        const textNodes = txBody.getElementsByTagNameNS(NS_A, 't');
        const content = Array.from(textNodes).map(t => t.textContent ?? '').join('');
        if (!content.trim()) continue;

        // Text formatting from first run
        const firstRPr = txBody.getElementsByTagNameNS(NS_A, 'rPr')[0];
        const szStr = firstRPr?.getAttribute('sz');
        // sz is hundredths of a pt; convert to px (1pt ≈ 1.333px)
        const fontSize = szStr ? Math.round(parseInt(szStr) / 100 * 1.333) : 24;
        const bold = firstRPr?.getAttribute('b') === '1';
        const italic = firstRPr?.getAttribute('i') === '1';
        const underline = firstRPr?.getAttribute('u') === 'sng';

        let color = '#1f2937';
        const solidFill = firstRPr?.getElementsByTagNameNS(NS_A, 'solidFill')[0];
        const srgbClr = solidFill?.getElementsByTagNameNS(NS_A, 'srgbClr')[0];
        if (srgbClr) color = `#${srgbClr.getAttribute('val') ?? '1f2937'}`;

        const firstPPr = txBody.getElementsByTagNameNS(NS_A, 'pPr')[0];
        const algn = firstPPr?.getAttribute('algn');
        const align: TextStyle['align'] = algn === 'ctr' ? 'center' : algn === 'r' ? 'right' : 'left';

        const xPct = (x / SLIDE_W_EMU) * 100;
        const yPct = (y / SLIDE_H_EMU) * 100;
        const wPct = (cx / SLIDE_W_EMU) * 100;
        const hPct = (cy / SLIDE_H_EMU) * 100;

        if (wPct <= 0 || hPct <= 0) continue;

        elements.push({
          id: uid(),
          type: 'text',
          x: Math.max(0, xPct),
          y: Math.max(0, yPct),
          w: Math.min(100, wPct),
          h: Math.min(100, hPct),
          content: content.trim(),
          style: {
            fontSize: Math.max(8, Math.min(120, fontSize)),
            bold,
            italic,
            underline,
            color,
            align,
            fontFamily: 'Inter',
          },
        });
      }
    }

    // Parse notes
    let notes = '';
    const slideNum = slidePath.match(/(\d+)\.xml$/)?.[1];
    if (slideNum) {
      const notesPath = `ppt/notesSlides/notesSlide${slideNum}.xml`;
      if (zip.files[notesPath]) {
        const notesXml = await zip.files[notesPath].async('text');
        const notesDoc = parser.parseFromString(notesXml, 'application/xml');
        const noteSpNodes = Array.from(notesDoc.getElementsByTagNameNS(NS_P, 'sp'));
        for (const noteSp of noteSpNodes.slice(1)) {
          const txBody = noteSp.getElementsByTagNameNS(NS_P, 'txBody')[0];
          if (!txBody) continue;
          const tNodes = txBody.getElementsByTagNameNS(NS_A, 't');
          const text = Array.from(tNodes).map(t => t.textContent ?? '').join('');
          if (text.trim()) { notes = text.trim(); break; }
        }
      }
    }

    slides.push({
      id: uid(),
      background: { type: 'color', value: bgColor },
      elements,
      notes,
      transition: 'fade',
    });
  }

  return {
    slides: slides.length > 0 ? slides : makeDefaultPresentation().slides,
    theme: THEMES[0],
    master: makeDefaultMaster(),
  };
}

// ── Animation helper ──────────────────────────────────────────────────────────

function getAnimationStyle(anim: ElementAnimation | undefined): React.CSSProperties | undefined {
  if (!anim || anim.type === 'none') return undefined;
  let keyframe = '';
  if (anim.type === 'fade') {
    keyframe = 'slideAnimFade';
  } else if (anim.type === 'zoom') {
    keyframe = 'slideAnimZoom';
  } else if (anim.type === 'fly-in') {
    const dir = anim.direction ?? 'left';
    keyframe =
      dir === 'left' ? 'slideAnimFlyLeft' :
      dir === 'right' ? 'slideAnimFlyRight' :
      dir === 'top' ? 'slideAnimFlyTop' :
      'slideAnimFlyBottom';
  }
  return {
    animation: `${keyframe} ${anim.duration}ms ease-out ${anim.delay}ms both`,
  };
}

// ── Save status ──────────────────────────────────────────────────────────────

type SaveStatus = 'saved' | 'saving' | 'unsaved' | 'error';

// ── Main component ───────────────────────────────────────────────────────────

export function SlideEditor() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const queryClient = useQueryClient();
  const slideId = searchParams.get('id') ?? '';

  const [title, setTitle] = useState('');
  const [presentation, setPresentation] = useState<SlidePresentation>(makeDefaultPresentation);
  const [selectedSlideIdx, setSelectedSlideIdx] = useState(0);
  const [selectedElementId, setSelectedElementId] = useState<string | null>(null);
  const [editingElementId, setEditingElementId] = useState<string | null>(null);
  const [saveStatus, setSaveStatus] = useState<SaveStatus>('saved');
  const [presenterMode, setPresenterMode] = useState(false);
  const [exportOpen, setExportOpen] = useState(false);
  const [themeOpen, setThemeOpen] = useState(false);
  const [masterMode, setMasterMode] = useState(false);
  const [showNotes, setShowNotes] = useState(true);
  const [importError, setImportError] = useState<string | null>(null);
  const [dragOverIdx, setDragOverIdx] = useState<number | null>(null);

  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastSavedRef = useRef('');
  const exportRef = useRef<HTMLDivElement>(null);
  const themeRef = useRef<HTMLDivElement>(null);
  const importInputRef = useRef<HTMLInputElement>(null);
  const dragSrcIdx = useRef<number | null>(null);

  const { isLoading: metaLoading, data: slideData } = useQuery({
    queryKey: ['slide', slideId],
    queryFn: () => slidesApi.getSlide(slideId),
    enabled: !!slideId,
    staleTime: 30_000,
  });

  const { isLoading: contentLoading, data: slideContent } = useQuery({
    queryKey: ['slide-content', slideId],
    queryFn: () => driveReadContent(slideData!.contentUrl),
    enabled: !!slideData?.contentUrl,
    staleTime: 30_000,
  });

  const isLoading = metaLoading || contentLoading;

  useEffect(() => {
    if (!slideData) return;
    setTitle(slideData.title);
  }, [slideData]);

  useEffect(() => {
    if (!slideContent) return;
    try {
      const parsed: SlidePresentation = JSON.parse(slideContent);
      if (parsed?.slides?.length > 0) {
        setPresentation(parsed);
        lastSavedRef.current = slideContent;
      }
    } catch {
      // keep default
    }
  }, [slideContent]);

  const contentMutation = useMutation({
    mutationFn: (content: string) =>
      driveWriteContent(slideData!.contentWriteUrl, content, 'slide.json'),
    onMutate: () => setSaveStatus('saving'),
    onSuccess: (_, content) => {
      setSaveStatus('saved');
      lastSavedRef.current = content;
      queryClient.invalidateQueries({ queryKey: ['slides'] });
    },
    onError: () => setSaveStatus('error'),
  });

  const metaMutation = useMutation({
    mutationFn: (newTitle: string) =>
      slidesApi.saveSlide(slideId, { title: newTitle }),
  });

  const scheduleAutoSave = useCallback((pres: SlidePresentation) => {
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    setSaveStatus('unsaved');
    saveTimerRef.current = setTimeout(() => {
      const content = JSON.stringify(pres);
      contentMutation.mutate(content);
    }, 2000);
  }, [contentMutation]);

  function updatePresentation(updater: (p: SlidePresentation) => SlidePresentation) {
    setPresentation((prev) => {
      const next = updater(prev);
      scheduleAutoSave(next);
      return next;
    });
  }

  function handleTitleBlur() {
    const trimmed = title.trim();
    if (!trimmed) return;
    metaMutation.mutate(trimmed);
  }

  // Close dropdowns on outside click
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (exportRef.current && !exportRef.current.contains(e.target as Node)) setExportOpen(false);
      if (themeRef.current && !themeRef.current.contains(e.target as Node)) setThemeOpen(false);
    }
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  useEffect(() => {
    return () => { if (saveTimerRef.current) clearTimeout(saveTimerRef.current); };
  }, []);

  // ── Slide operations ─────────────────────────────────────────────────────

  const currentSlide = presentation.slides[selectedSlideIdx] ?? presentation.slides[0];

  function addSlide() {
    const master = presentation.master ?? makeDefaultMaster();
    const newSlide: Slide = {
      id: uid(),
      background: { type: 'color', value: master.background },
      elements: [],
      notes: '',
      transition: 'fade',
    };
    updatePresentation((p) => {
      const slides = [...p.slides];
      slides.splice(selectedSlideIdx + 1, 0, newSlide);
      return { ...p, slides };
    });
    setSelectedSlideIdx(selectedSlideIdx + 1);
    setSelectedElementId(null);
  }

  function duplicateSlide() {
    const copy: Slide = {
      ...currentSlide,
      id: uid(),
      elements: currentSlide.elements.map((el) => ({ ...el, id: uid() })),
    };
    updatePresentation((p) => {
      const slides = [...p.slides];
      slides.splice(selectedSlideIdx + 1, 0, copy);
      return { ...p, slides };
    });
    setSelectedSlideIdx(selectedSlideIdx + 1);
  }

  function deleteSlide() {
    if (presentation.slides.length <= 1) return;
    updatePresentation((p) => {
      const slides = p.slides.filter((_, i) => i !== selectedSlideIdx);
      return { ...p, slides };
    });
    setSelectedSlideIdx(Math.max(0, selectedSlideIdx - 1));
    setSelectedElementId(null);
  }

  function moveSlide(dir: -1 | 1) {
    const newIdx = selectedSlideIdx + dir;
    if (newIdx < 0 || newIdx >= presentation.slides.length) return;
    updatePresentation((p) => {
      const slides = [...p.slides];
      [slides[selectedSlideIdx], slides[newIdx]] = [slides[newIdx], slides[selectedSlideIdx]];
      return { ...p, slides };
    });
    setSelectedSlideIdx(newIdx);
  }

  // ── Drag-to-reorder ──────────────────────────────────────────────────────

  function handleSlideDragStart(e: React.DragEvent, idx: number) {
    dragSrcIdx.current = idx;
    e.dataTransfer.effectAllowed = 'move';
    // Minimal ghost — browser default thumbnail is fine
  }

  function handleSlideDragOver(e: React.DragEvent, idx: number) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverIdx(idx);
  }

  function handleSlideDrop(e: React.DragEvent, dropIdx: number) {
    e.preventDefault();
    const srcIdx = dragSrcIdx.current;
    if (srcIdx === null || srcIdx === dropIdx) {
      setDragOverIdx(null);
      return;
    }
    updatePresentation((p) => {
      const slides = [...p.slides];
      const [removed] = slides.splice(srcIdx, 1);
      slides.splice(dropIdx, 0, removed);
      return { ...p, slides };
    });
    // Keep the dragged slide selected
    setSelectedSlideIdx(dropIdx);
    setSelectedElementId(null);
    dragSrcIdx.current = null;
    setDragOverIdx(null);
  }

  function handleSlideDragEnd() {
    dragSrcIdx.current = null;
    setDragOverIdx(null);
  }

  function updateCurrentSlide(updater: (s: Slide) => Slide) {
    updatePresentation((p) => {
      const slides = p.slides.map((s, i) => i === selectedSlideIdx ? updater(s) : s);
      return { ...p, slides };
    });
  }

  // ── Element operations ───────────────────────────────────────────────────

  function addTextBox() {
    const el: TextElement = {
      id: uid(),
      type: 'text',
      x: 20, y: 40, w: 60, h: 15,
      content: 'New text box',
      style: { fontSize: 24, bold: false, italic: false, underline: false, color: presentation.theme.textColor, align: 'left', fontFamily: 'Inter' },
    };
    updateCurrentSlide((s) => ({ ...s, elements: [...s.elements, el] }));
    setSelectedElementId(el.id);
  }

  function addShape(shape: 'rect' | 'circle') {
    const el: ShapeElement = {
      id: uid(),
      type: 'shape',
      shape,
      x: 30, y: 35, w: 40, h: 25,
      fill: presentation.theme.primaryColor,
      stroke: 'transparent',
      strokeWidth: 0,
    };
    updateCurrentSlide((s) => ({ ...s, elements: [...s.elements, el] }));
    setSelectedElementId(el.id);
  }

  function deleteElement(elementId: string) {
    updateCurrentSlide((s) => ({ ...s, elements: s.elements.filter((e) => e.id !== elementId) }));
    setSelectedElementId(null);
  }

  function updateElement(elementId: string, updater: (el: SlideElement) => SlideElement) {
    updateCurrentSlide((s) => ({
      ...s,
      elements: s.elements.map((e) => e.id === elementId ? updater(e) : e),
    }));
  }

  function updateTextStyle(elementId: string, style: Partial<TextStyle>) {
    updateElement(elementId, (el) => {
      if (el.type !== 'text') return el;
      return { ...el, style: { ...el.style, ...style } };
    });
  }

  function updateElementAnimation(elementId: string, anim: Partial<ElementAnimation>) {
    updateElement(elementId, (el) => {
      const current: ElementAnimation = el.animation ?? { type: 'none', duration: 500, delay: 0 };
      return { ...el, animation: { ...current, ...anim } };
    });
  }

  // ── Slide master operations ──────────────────────────────────────────────

  function updateMaster(updater: (m: SlideMaster) => SlideMaster) {
    updatePresentation((p) => ({
      ...p,
      master: updater(p.master ?? makeDefaultMaster()),
    }));
  }

  function applyMasterToAllSlides() {
    const master = presentation.master ?? makeDefaultMaster();
    updatePresentation((p) => ({
      ...p,
      slides: p.slides.map((s) => ({
        ...s,
        background: { type: 'color', value: master.background },
        elements: s.elements.map((el, idx) => {
          if (el.type !== 'text') return el;
          // Apply title style to first text element, body style to rest
          if (idx === 0) {
            return {
              ...el,
              style: {
                ...el.style,
                fontSize: master.titleFontSize,
                bold: master.titleBold,
                color: master.titleColor,
              },
            };
          }
          return {
            ...el,
            style: {
              ...el.style,
              fontSize: master.bodyFontSize,
              bold: master.bodyBold,
              color: master.bodyColor,
            },
          };
        }),
      })),
    }));
  }

  // ── PPTX import handler ───────────────────────────────────────────────────

  async function handleImportPptx(file: File) {
    setImportError(null);
    try {
      const imported = await importFromPptx(file);
      setPresentation(imported);
      scheduleAutoSave(imported);
    } catch (err) {
      setImportError(err instanceof Error ? err.message : 'Failed to import file');
    }
  }

  // ── Selected element ─────────────────────────────────────────────────────

  const selectedElement = currentSlide?.elements.find((e) => e.id === selectedElementId) ?? null;

  const saveStatusText =
    saveStatus === 'saving' ? 'Saving…' :
    saveStatus === 'unsaved' ? 'Unsaved changes' :
    saveStatus === 'error' ? 'Save failed' :
    'All changes saved';

  const saveStatusClass =
    saveStatus === 'saving' ? styles.saveStatusSaving :
    saveStatus === 'error' ? styles.saveStatusError :
    '';

  if (isLoading) return <div style={{ padding: '2rem' }}>Loading presentation…</div>;

  // ── Presenter mode ───────────────────────────────────────────────────────

  if (presenterMode) {
    return (
      <PresenterView
        presentation={presentation}
        onExit={() => setPresenterMode(false)}
      />
    );
  }

  const master = presentation.master ?? makeDefaultMaster();

  return (
    <div className={styles.editorWrapper}>
      {/* Hidden PPTX file input */}
      <input
        ref={importInputRef}
        type="file"
        accept=".pptx,application/vnd.openxmlformats-officedocument.presentationml.presentation"
        style={{ display: 'none' }}
        onChange={(e) => {
          const file = e.target.files?.[0];
          if (file) handleImportPptx(file);
          e.target.value = '';
        }}
      />

      {/* Top bar */}
      <div className={styles.topBar}>
        <Button variant="ghost" icon={<ArrowLeft size={16} />} onClick={() => router.push('/slides')} className={styles.backBtn}>
          Slides
        </Button>

        <div className={styles.titleArea}>
          <Presentation size={18} color="var(--color-rose, #e11d48)" />
          <input
            className={styles.titleInput}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            onBlur={handleTitleBlur}
            onKeyDown={(e) => { if (e.key === 'Enter') e.currentTarget.blur(); }}
            placeholder="Untitled presentation"
          />
        </div>

        <span className={`${styles.saveStatus} ${saveStatusClass}`}>{saveStatusText}</span>

        <div className={styles.actions}>
          {/* Master toggle */}
          <Button
            variant={masterMode ? 'primary' : 'secondary'}
            icon={<LayoutTemplate size={16} />}
            onClick={() => { setMasterMode((v) => !v); setSelectedElementId(null); }}
            title="Slide Master"
          >
            Master
          </Button>

          {/* Theme picker */}
          <div className={styles.dropdownTrigger} ref={themeRef}>
            <Button variant="secondary" onClick={() => setThemeOpen((v) => !v)}>
              Theme <ChevronDown size={14} />
            </Button>
            {themeOpen && (
              <div className={`${styles.dropdownMenu} ${styles.themeMenu}`}>
                {THEMES.map((theme) => (
                  <button
                    key={theme.name}
                    className={`${styles.themeOption} ${presentation.theme.name === theme.name ? styles.themeOptionActive : ''}`}
                    onClick={() => {
                      updatePresentation((p) => ({
                        ...p,
                        theme,
                        slides: p.slides.map((s) => ({
                          ...s,
                          background: { type: 'color', value: theme.backgroundColor },
                          elements: s.elements.map((el) => {
                            if (el.type === 'text') return { ...el, style: { ...el.style, color: theme.textColor } };
                            if (el.type === 'shape') return { ...el, fill: theme.primaryColor };
                            return el;
                          }),
                        })),
                      }));
                      setThemeOpen(false);
                    }}
                  >
                    <span
                      className={styles.themeColorDot}
                      style={{ background: theme.primaryColor }}
                    />
                    {theme.name}
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Export + Import */}
          <div className={styles.dropdownTrigger} ref={exportRef}>
            <Button variant="secondary" icon={<Download size={16} />} onClick={() => setExportOpen((v) => !v)}>
              Export <ChevronDown size={14} />
            </Button>
            {exportOpen && (
              <div className={styles.dropdownMenu}>
                <button
                  className={styles.dropdownItem}
                  onClick={async () => {
                    setExportOpen(false);
                    await exportAsPptx(title || 'presentation', presentation);
                  }}
                >
                  PowerPoint (.pptx)
                </button>
              </div>
            )}
          </div>

          <Button
            variant="secondary"
            icon={<Upload size={16} />}
            onClick={() => importInputRef.current?.click()}
            title="Import PPTX"
          >
            Import
          </Button>

          <Button icon={<Play size={16} />} onClick={() => setPresenterMode(true)}>
            Present
          </Button>
        </div>
      </div>

      {/* Import error banner */}
      {importError && (
        <div className={styles.errorBanner}>
          {importError}
          <button onClick={() => setImportError(null)} className={styles.errorBannerClose}>✕</button>
        </div>
      )}

      {/* Toolbar */}
      <div className={styles.toolbar}>
        <button className={styles.toolbarBtn} onClick={addTextBox} title="Add text box">
          <Type size={16} /> Text
        </button>
        <button className={styles.toolbarBtn} onClick={() => addShape('rect')} title="Add rectangle">
          <Square size={16} /> Rectangle
        </button>
        <button className={styles.toolbarBtn} onClick={() => addShape('circle')} title="Add circle">
          <Circle size={16} /> Circle
        </button>

        {selectedElement?.type === 'text' && (
          <>
            <div className={styles.toolbarDivider} />
            <button
              className={`${styles.toolbarBtn} ${(selectedElement as TextElement).style.bold ? styles.toolbarBtnActive : ''}`}
              onClick={() => updateTextStyle(selectedElement.id, { bold: !(selectedElement as TextElement).style.bold })}
              title="Bold"
            >
              <Bold size={16} />
            </button>
            <button
              className={`${styles.toolbarBtn} ${(selectedElement as TextElement).style.italic ? styles.toolbarBtnActive : ''}`}
              onClick={() => updateTextStyle(selectedElement.id, { italic: !(selectedElement as TextElement).style.italic })}
              title="Italic"
            >
              <Italic size={16} />
            </button>
            <button
              className={`${styles.toolbarBtn} ${(selectedElement as TextElement).style.underline ? styles.toolbarBtnActive : ''}`}
              onClick={() => updateTextStyle(selectedElement.id, { underline: !(selectedElement as TextElement).style.underline })}
              title="Underline"
            >
              <Underline size={16} />
            </button>
            <div className={styles.toolbarDivider} />
            <button className={styles.toolbarBtn} onClick={() => updateTextStyle(selectedElement.id, { fontSize: Math.max(8, (selectedElement as TextElement).style.fontSize - 2) })} title="Decrease font size">
              <Minus size={14} />
            </button>
            <span className={styles.toolbarFontSize}>{(selectedElement as TextElement).style.fontSize}px</span>
            <button className={styles.toolbarBtn} onClick={() => updateTextStyle(selectedElement.id, { fontSize: Math.min(120, (selectedElement as TextElement).style.fontSize + 2) })} title="Increase font size">
              <Plus size={14} />
            </button>
            <div className={styles.toolbarDivider} />
            <button
              className={`${styles.toolbarBtn} ${(selectedElement as TextElement).style.align === 'left' ? styles.toolbarBtnActive : ''}`}
              onClick={() => updateTextStyle(selectedElement.id, { align: 'left' })}
              title="Align left"
            >
              <AlignLeft size={16} />
            </button>
            <button
              className={`${styles.toolbarBtn} ${(selectedElement as TextElement).style.align === 'center' ? styles.toolbarBtnActive : ''}`}
              onClick={() => updateTextStyle(selectedElement.id, { align: 'center' })}
              title="Align center"
            >
              <AlignCenter size={16} />
            </button>
            <button
              className={`${styles.toolbarBtn} ${(selectedElement as TextElement).style.align === 'right' ? styles.toolbarBtnActive : ''}`}
              onClick={() => updateTextStyle(selectedElement.id, { align: 'right' })}
              title="Align right"
            >
              <AlignRight size={16} />
            </button>
            <div className={styles.toolbarDivider} />
            <input
              type="color"
              className={styles.colorPicker}
              value={(selectedElement as TextElement).style.color}
              onChange={(e) => updateTextStyle(selectedElement.id, { color: e.target.value })}
              title="Text color"
            />
            <button
              className={styles.toolbarBtn}
              onClick={() => deleteElement(selectedElement.id)}
              title="Delete element"
            >
              <Trash2 size={16} />
            </button>
          </>
        )}

        {selectedElement?.type === 'shape' && (
          <>
            <div className={styles.toolbarDivider} />
            <input
              type="color"
              className={styles.colorPicker}
              value={(selectedElement as ShapeElement).fill}
              onChange={(e) => updateElement(selectedElement.id, (el) => ({ ...el, fill: e.target.value } as ShapeElement))}
              title="Fill color"
            />
            <button
              className={styles.toolbarBtn}
              onClick={() => deleteElement(selectedElement.id)}
              title="Delete element"
            >
              <Trash2 size={16} />
            </button>
          </>
        )}

        {/* Animation controls for selected element */}
        {selectedElement && (
          <>
            <div className={styles.toolbarDivider} />
            <Zap size={14} style={{ color: 'var(--color-text-muted)', flexShrink: 0 }} />
            <select
              className={styles.toolbarSelect}
              value={selectedElement.animation?.type ?? 'none'}
              onChange={(e) =>
                updateElementAnimation(selectedElement.id, { type: e.target.value as ElementAnimation['type'] })
              }
              title="Entry animation"
            >
              <option value="none">No animation</option>
              <option value="fade">Fade in</option>
              <option value="fly-in">Fly in</option>
              <option value="zoom">Zoom in</option>
            </select>
            {selectedElement.animation?.type === 'fly-in' && (
              <select
                className={styles.toolbarSelect}
                value={selectedElement.animation.direction ?? 'left'}
                onChange={(e) =>
                  updateElementAnimation(selectedElement.id, { direction: e.target.value as ElementAnimation['direction'] })
                }
                title="Direction"
              >
                <option value="left">From left</option>
                <option value="right">From right</option>
                <option value="top">From top</option>
                <option value="bottom">From bottom</option>
              </select>
            )}
            {selectedElement.animation && selectedElement.animation.type !== 'none' && (
              <>
                <span className={styles.toolbarLabel} title="Duration (ms)">
                  Duration
                  <input
                    type="number"
                    min={100}
                    max={2000}
                    step={100}
                    value={selectedElement.animation.duration}
                    onChange={(e) =>
                      updateElementAnimation(selectedElement.id, { duration: parseInt(e.target.value) || 500 })
                    }
                    className={styles.toolbarNumberInput}
                    title="Duration in milliseconds"
                  />
                  ms
                </span>
                <span className={styles.toolbarLabel} title="Delay (ms)">
                  Delay
                  <input
                    type="number"
                    min={0}
                    max={2000}
                    step={100}
                    value={selectedElement.animation.delay}
                    onChange={(e) =>
                      updateElementAnimation(selectedElement.id, { delay: parseInt(e.target.value) || 0 })
                    }
                    className={styles.toolbarNumberInput}
                    title="Delay in milliseconds"
                  />
                  ms
                </span>
              </>
            )}
          </>
        )}

        {/* Background color */}
        <div className={styles.toolbarDivider} />
        <label className={styles.toolbarLabel} title="Slide background color">
          BG
          <input
            type="color"
            className={styles.colorPicker}
            value={currentSlide.background.value}
            onChange={(e) => updateCurrentSlide((s) => ({ ...s, background: { type: 'color', value: e.target.value } }))}
          />
        </label>

        {/* Transition */}
        <div className={styles.toolbarDivider} />
        <select
          className={styles.toolbarSelect}
          value={currentSlide.transition}
          onChange={(e) => updateCurrentSlide((s) => ({ ...s, transition: e.target.value as Slide['transition'] }))}
          title="Slide transition"
        >
          <option value="none">No transition</option>
          <option value="fade">Fade</option>
          <option value="slide">Slide</option>
          <option value="zoom">Zoom</option>
        </select>
      </div>

      {/* Main area */}
      <div className={styles.mainArea}>
        {/* Slide panel */}
        <div className={styles.slidePanel}>
          <div className={styles.slidePanelHeader}>
            <span>Slides ({presentation.slides.length})</span>
            <button className={styles.slidePanelBtn} onClick={addSlide} title="Add slide"><Plus size={14} /></button>
          </div>
          <div className={styles.slidePanelList}>
            {presentation.slides.map((slide, idx) => (
              <div
                key={slide.id}
                draggable
                className={[
                  styles.slideThumbnail,
                  idx === selectedSlideIdx ? styles.slideThumbnailActive : '',
                  dragOverIdx === idx && dragSrcIdx.current !== idx ? styles.slideThumbnailDropTarget : '',
                ].join(' ')}
                onClick={() => { setSelectedSlideIdx(idx); setSelectedElementId(null); }}
                onDragStart={(e) => handleSlideDragStart(e, idx)}
                onDragOver={(e) => handleSlideDragOver(e, idx)}
                onDrop={(e) => handleSlideDrop(e, idx)}
                onDragEnd={handleSlideDragEnd}
              >
                <span className={styles.slideThumbnailNum}>{idx + 1}</span>
                <SlideThumbnail slide={slide} />
              </div>
            ))}
          </div>
          <div className={styles.slidePanelFooter}>
            <button className={styles.slidePanelBtn} onClick={() => moveSlide(-1)} disabled={selectedSlideIdx === 0} title="Move up"><ChevronUp size={14} /></button>
            <button className={styles.slidePanelBtn} onClick={() => moveSlide(1)} disabled={selectedSlideIdx >= presentation.slides.length - 1} title="Move down"><ChevronDown size={14} /></button>
            <button className={styles.slidePanelBtn} onClick={duplicateSlide} title="Duplicate slide"><Copy size={14} /></button>
            <button className={styles.slidePanelBtn} onClick={deleteSlide} disabled={presentation.slides.length <= 1} title="Delete slide"><Trash2 size={14} /></button>
          </div>
        </div>

        {/* Canvas area */}
        <div className={styles.canvasArea}>
          {currentSlide && (
            <SlideCanvas
              slide={currentSlide}
              selectedElementId={selectedElementId}
              editingElementId={editingElementId}
              onSelectElement={setSelectedElementId}
              onStartEdit={setEditingElementId}
              onStopEdit={() => setEditingElementId(null)}
              onUpdateElement={updateElement}
              onClickBackground={() => { setSelectedElementId(null); setEditingElementId(null); }}
            />
          )}
        </div>

        {/* Right panel: notes or master settings */}
        <div className={styles.rightPanel}>
          {masterMode ? (
            <>
              <div className={styles.rightPanelHeader}>
                <span>Slide Master</span>
              </div>
              <div className={styles.masterPanel}>
                <div className={styles.masterSection}>
                  <label className={styles.masterLabel}>Background</label>
                  <div className={styles.masterRow}>
                    <input
                      type="color"
                      className={styles.colorPicker}
                      value={master.background}
                      onChange={(e) => updateMaster((m) => ({ ...m, background: e.target.value }))}
                    />
                    <span className={styles.masterColorVal}>{master.background}</span>
                  </div>
                </div>

                <div className={styles.masterSection}>
                  <label className={styles.masterLabel}>Title Style</label>
                  <div className={styles.masterRow}>
                    <span className={styles.masterFieldLabel}>Size</span>
                    <input
                      type="number"
                      min={10}
                      max={120}
                      value={master.titleFontSize}
                      onChange={(e) => updateMaster((m) => ({ ...m, titleFontSize: parseInt(e.target.value) || 40 }))}
                      className={styles.masterNumberInput}
                    />
                  </div>
                  <div className={styles.masterRow}>
                    <span className={styles.masterFieldLabel}>Bold</span>
                    <input
                      type="checkbox"
                      checked={master.titleBold}
                      onChange={(e) => updateMaster((m) => ({ ...m, titleBold: e.target.checked }))}
                    />
                  </div>
                  <div className={styles.masterRow}>
                    <span className={styles.masterFieldLabel}>Color</span>
                    <input
                      type="color"
                      className={styles.colorPicker}
                      value={master.titleColor}
                      onChange={(e) => updateMaster((m) => ({ ...m, titleColor: e.target.value }))}
                    />
                  </div>
                </div>

                <div className={styles.masterSection}>
                  <label className={styles.masterLabel}>Body Style</label>
                  <div className={styles.masterRow}>
                    <span className={styles.masterFieldLabel}>Size</span>
                    <input
                      type="number"
                      min={8}
                      max={80}
                      value={master.bodyFontSize}
                      onChange={(e) => updateMaster((m) => ({ ...m, bodyFontSize: parseInt(e.target.value) || 24 }))}
                      className={styles.masterNumberInput}
                    />
                  </div>
                  <div className={styles.masterRow}>
                    <span className={styles.masterFieldLabel}>Bold</span>
                    <input
                      type="checkbox"
                      checked={master.bodyBold}
                      onChange={(e) => updateMaster((m) => ({ ...m, bodyBold: e.target.checked }))}
                    />
                  </div>
                  <div className={styles.masterRow}>
                    <span className={styles.masterFieldLabel}>Color</span>
                    <input
                      type="color"
                      className={styles.colorPicker}
                      value={master.bodyColor}
                      onChange={(e) => updateMaster((m) => ({ ...m, bodyColor: e.target.value }))}
                    />
                  </div>
                </div>

                <button className={styles.masterApplyBtn} onClick={applyMasterToAllSlides}>
                  Apply to All Slides
                </button>
                <p className={styles.masterHint}>
                  New slides will use the master background. "Apply to All" updates backgrounds and text styles across all slides.
                </p>
              </div>
            </>
          ) : (
            <>
              <div className={styles.rightPanelHeader}>
                <span>Speaker Notes</span>
                <button className={styles.slidePanelBtn} onClick={() => setShowNotes((v) => !v)}>
                  {showNotes ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
                </button>
              </div>
              {showNotes && (
                <textarea
                  className={styles.notesArea}
                  placeholder="Add speaker notes for this slide…"
                  value={currentSlide?.notes ?? ''}
                  onChange={(e) => updateCurrentSlide((s) => ({ ...s, notes: e.target.value }))}
                />
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
}

// ── Slide canvas component ───────────────────────────────────────────────────

function SlideCanvas({
  slide,
  selectedElementId,
  editingElementId,
  onSelectElement,
  onStartEdit,
  onStopEdit,
  onUpdateElement,
  onClickBackground,
}: {
  slide: Slide;
  selectedElementId: string | null;
  editingElementId: string | null;
  onSelectElement: (id: string) => void;
  onStartEdit: (id: string) => void;
  onStopEdit: () => void;
  onUpdateElement: (id: string, updater: (el: SlideElement) => SlideElement) => void;
  onClickBackground: () => void;
}) {
  const canvasRef = useRef<HTMLDivElement>(null);
  const dragState = useRef<{
    elementId: string;
    startMouseX: number;
    startMouseY: number;
    startX: number;
    startY: number;
  } | null>(null);

  function handleMouseDown(e: React.MouseEvent, elementId: string, el: SlideElement) {
    if (editingElementId === elementId) return;
    e.stopPropagation();
    onSelectElement(elementId);

    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    dragState.current = {
      elementId,
      startMouseX: e.clientX,
      startMouseY: e.clientY,
      startX: el.x,
      startY: el.y,
    };

    function onMove(me: MouseEvent) {
      if (!dragState.current || !canvas) return;
      const dx = ((me.clientX - dragState.current.startMouseX) / rect.width) * 100;
      const dy = ((me.clientY - dragState.current.startMouseY) / rect.height) * 100;
      const newX = Math.max(0, Math.min(100, dragState.current.startX + dx));
      const newY = Math.max(0, Math.min(100, dragState.current.startY + dy));
      onUpdateElement(dragState.current.elementId, (el) => ({ ...el, x: newX, y: newY }));
    }

    function onUp() {
      dragState.current = null;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  return (
    <div
      ref={canvasRef}
      className={styles.slideCanvas}
      style={{ background: slide.background.value }}
      onClick={onClickBackground}
    >
      {slide.elements.map((el) => {
        const isSelected = el.id === selectedElementId;
        const isEditing = el.id === editingElementId;

        if (el.type === 'text') {
          return (
            <div
              key={el.id}
              className={`${styles.slideElement} ${isSelected ? styles.slideElementSelected : ''}`}
              style={{
                left: `${el.x}%`,
                top: `${el.y}%`,
                width: `${el.w}%`,
                height: `${el.h}%`,
                cursor: isEditing ? 'text' : 'move',
              }}
              onMouseDown={(e) => handleMouseDown(e, el.id, el)}
              onDoubleClick={(e) => { e.stopPropagation(); onStartEdit(el.id); }}
              onClick={(e) => e.stopPropagation()}
            >
              {isEditing ? (
                <textarea
                  className={styles.textEditArea}
                  autoFocus
                  defaultValue={el.content}
                  style={{
                    fontSize: `${el.style.fontSize * 0.75}px`,
                    fontWeight: el.style.bold ? 700 : 400,
                    fontStyle: el.style.italic ? 'italic' : 'normal',
                    textDecoration: el.style.underline ? 'underline' : 'none',
                    color: el.style.color,
                    textAlign: el.style.align,
                    fontFamily: el.style.fontFamily,
                  }}
                  onBlur={(e) => {
                    onUpdateElement(el.id, (elem) => ({ ...elem, content: e.target.value } as TextElement));
                    onStopEdit();
                  }}
                  onClick={(e) => e.stopPropagation()}
                />
              ) : (
                <div
                  className={styles.textDisplay}
                  style={{
                    fontSize: `${el.style.fontSize * 0.75}px`,
                    fontWeight: el.style.bold ? 700 : 400,
                    fontStyle: el.style.italic ? 'italic' : 'normal',
                    textDecoration: el.style.underline ? 'underline' : 'none',
                    color: el.style.color,
                    textAlign: el.style.align,
                    fontFamily: el.style.fontFamily,
                  }}
                >
                  {el.content || <span style={{ opacity: 0.4 }}>Empty text box</span>}
                </div>
              )}
            </div>
          );
        }

        if (el.type === 'shape') {
          return (
            <div
              key={el.id}
              className={`${styles.slideElement} ${isSelected ? styles.slideElementSelected : ''}`}
              style={{
                left: `${el.x}%`,
                top: `${el.y}%`,
                width: `${el.w}%`,
                height: `${el.h}%`,
                cursor: 'move',
              }}
              onMouseDown={(e) => handleMouseDown(e, el.id, el)}
              onClick={(e) => e.stopPropagation()}
            >
              <div
                style={{
                  width: '100%',
                  height: '100%',
                  background: el.fill,
                  border: el.strokeWidth > 0 ? `${el.strokeWidth}px solid ${el.stroke}` : 'none',
                  borderRadius: el.shape === 'circle' ? '50%' : '0',
                }}
              />
            </div>
          );
        }

        return null;
      })}
    </div>
  );
}

// ── Slide thumbnail ──────────────────────────────────────────────────────────

function SlideThumbnail({ slide }: { slide: Slide }) {
  return (
    <div className={styles.thumbnailPreview} style={{ background: slide.background.value }}>
      {slide.elements.map((el) => {
        if (el.type === 'text') {
          return (
            <div
              key={el.id}
              style={{
                position: 'absolute',
                left: `${el.x}%`,
                top: `${el.y}%`,
                width: `${el.w}%`,
                height: `${el.h}%`,
                fontSize: `${el.style.fontSize * 0.075}px`,
                fontWeight: el.style.bold ? 700 : 400,
                color: el.style.color,
                overflow: 'hidden',
                lineHeight: 1.2,
              }}
            >
              {el.content}
            </div>
          );
        }
        if (el.type === 'shape') {
          return (
            <div
              key={el.id}
              style={{
                position: 'absolute',
                left: `${el.x}%`,
                top: `${el.y}%`,
                width: `${el.w}%`,
                height: `${el.h}%`,
                background: el.fill,
                borderRadius: el.shape === 'circle' ? '50%' : '0',
              }}
            />
          );
        }
        return null;
      })}
    </div>
  );
}

// ── Presenter view ───────────────────────────────────────────────────────────

function PresenterView({ presentation, onExit }: { presentation: SlidePresentation; onExit: () => void }) {
  const [idx, setIdx] = useState(0);
  const [animKey, setAnimKey] = useState(0);
  const slide = presentation.slides[idx];
  const total = presentation.slides.length;

  function next() {
    setIdx((i) => {
      const next = Math.min(total - 1, i + 1);
      if (next !== i) setAnimKey((k) => k + 1);
      return next;
    });
  }
  function prev() {
    setIdx((i) => {
      const next = Math.max(0, i - 1);
      if (next !== i) setAnimKey((k) => k + 1);
      return next;
    });
  }

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') next();
      else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') prev();
      else if (e.key === 'Escape') onExit();
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onExit]);

  const nextSlide = presentation.slides[idx + 1];

  return (
    <div className={styles.presenterWrapper}>
      {/* Current slide */}
      <div className={styles.presenterMain}>
        <div className={styles.presenterSlide} style={{ background: slide.background.value }}>
          {slide.elements.map((el) => {
            const animStyle = getAnimationStyle(el.animation);
            if (el.type === 'text') {
              return (
                <div
                  key={`${el.id}-${animKey}`}
                  style={{
                    position: 'absolute',
                    left: `${el.x}%`,
                    top: `${el.y}%`,
                    width: `${el.w}%`,
                    height: `${el.h}%`,
                    fontSize: `${el.style.fontSize * 0.75}px`,
                    fontWeight: el.style.bold ? 700 : 400,
                    fontStyle: el.style.italic ? 'italic' : 'normal',
                    textDecoration: el.style.underline ? 'underline' : 'none',
                    color: el.style.color,
                    textAlign: el.style.align,
                    fontFamily: el.style.fontFamily,
                    overflow: 'hidden',
                    ...animStyle,
                  }}
                >
                  {el.content}
                </div>
              );
            }
            if (el.type === 'shape') {
              return (
                <div
                  key={`${el.id}-${animKey}`}
                  style={{
                    position: 'absolute',
                    left: `${el.x}%`,
                    top: `${el.y}%`,
                    width: `${el.w}%`,
                    height: `${el.h}%`,
                    background: el.fill,
                    borderRadius: el.shape === 'circle' ? '50%' : '0',
                    ...animStyle,
                  }}
                />
              );
            }
            return null;
          })}
        </div>

        <div className={styles.presenterControls}>
          <button className={styles.presenterBtn} onClick={prev} disabled={idx === 0}>←</button>
          <span className={styles.presenterCounter}>{idx + 1} / {total}</span>
          <button className={styles.presenterBtn} onClick={next} disabled={idx === total - 1}>→</button>
          <button className={styles.presenterBtnExit} onClick={onExit}>✕ Exit</button>
        </div>
      </div>

      {/* Right panel: notes + next slide */}
      <div className={styles.presenterSidebar}>
        {nextSlide && (
          <div className={styles.presenterNextSection}>
            <div className={styles.presenterSideLabel}>Next slide</div>
            <div className={styles.presenterNextSlide} style={{ background: nextSlide.background.value }}>
              {nextSlide.elements.map((el) => {
                if (el.type === 'text') {
                  return (
                    <div key={el.id} style={{ position: 'absolute', left: `${el.x}%`, top: `${el.y}%`, width: `${el.w}%`, height: `${el.h}%`, fontSize: `${el.style.fontSize * 0.3}px`, fontWeight: el.style.bold ? 700 : 400, color: el.style.color, overflow: 'hidden' }}>
                      {el.content}
                    </div>
                  );
                }
                if (el.type === 'shape') {
                  return <div key={el.id} style={{ position: 'absolute', left: `${el.x}%`, top: `${el.y}%`, width: `${el.w}%`, height: `${el.h}%`, background: el.fill, borderRadius: el.shape === 'circle' ? '50%' : '0' }} />;
                }
                return null;
              })}
            </div>
          </div>
        )}

        <div className={styles.presenterNotesSection}>
          <div className={styles.presenterSideLabel}>Speaker notes</div>
          <div className={styles.presenterNotes}>
            {slide.notes || <span style={{ opacity: 0.4 }}>No notes for this slide</span>}
          </div>
        </div>
      </div>
    </div>
  );
}
