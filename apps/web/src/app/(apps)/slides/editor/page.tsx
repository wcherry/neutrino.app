import { Suspense } from 'react';
import { SlideEditor } from './SlideEditor';

export default function SlideEditorPage() {
  return (
    <Suspense fallback={<div style={{ padding: '2rem' }}>Loading presentation…</div>}>
      <SlideEditor />
    </Suspense>
  );
}
