'use client';

import React, { useState } from 'react';
import { X, Sparkles, SpellCheck, Languages, PenLine, BookOpen } from 'lucide-react';
import { docsAI, type GrammarIssue } from '@/lib/api';
import styles from './AIPanel.module.css';

interface AIPanelProps {
  docId: string;
  onClose: () => void;
  onInsertText?: (text: string) => void;
  getDocContent?: () => string;
}

const LANGUAGES = [
  { code: 'Spanish', label: 'Spanish' },
  { code: 'French', label: 'French' },
  { code: 'German', label: 'German' },
  { code: 'Portuguese', label: 'Portuguese' },
  { code: 'Italian', label: 'Italian' },
  { code: 'Japanese', label: 'Japanese' },
  { code: 'Chinese (Simplified)', label: 'Chinese (Simplified)' },
  { code: 'Korean', label: 'Korean' },
  { code: 'Arabic', label: 'Arabic' },
  { code: 'Russian', label: 'Russian' },
];

export function AIPanel({ docId, onClose, onInsertText, getDocContent }: AIPanelProps) {
  // Help me write
  const [writeDesc, setWriteDesc] = useState('');
  const [writeResult, setWriteResult] = useState('');
  const [writeLoading, setWriteLoading] = useState(false);
  const [writeError, setWriteError] = useState('');

  // Grammar check
  const [grammarIssues, setGrammarIssues] = useState<GrammarIssue[] | null>(null);
  const [grammarLoading, setGrammarLoading] = useState(false);
  const [grammarError, setGrammarError] = useState('');
  const [grammarChecked, setGrammarChecked] = useState(false);

  // Translate
  const [translateLang, setTranslateLang] = useState('Spanish');
  const [translateResult, setTranslateResult] = useState('');
  const [translateLoading, setTranslateLoading] = useState(false);
  const [translateError, setTranslateError] = useState('');

  // Summarize
  const [summary, setSummary] = useState('');
  const [summarizeLoading, setSummarizeLoading] = useState(false);
  const [summarizeError, setSummarizeError] = useState('');

  const handleHelpMeWrite = async () => {
    if (!writeDesc.trim()) return;
    setWriteLoading(true);
    setWriteError('');
    setWriteResult('');
    try {
      const res = await docsAI.helpMeWrite(writeDesc.trim());
      setWriteResult(res.completion);
    } catch (e) {
      setWriteError(e instanceof Error ? e.message : 'AI request failed');
    } finally {
      setWriteLoading(false);
    }
  };

  const handleGrammarCheck = async () => {
    const content = getDocContent?.() ?? '';
    if (!content.trim()) {
      setGrammarError('No document content to check');
      return;
    }
    setGrammarLoading(true);
    setGrammarError('');
    setGrammarIssues(null);
    setGrammarChecked(false);
    try {
      const res = await docsAI.grammarCheck(docId, content);
      setGrammarIssues(res.issues);
      setGrammarChecked(true);
    } catch (e) {
      setGrammarError(e instanceof Error ? e.message : 'Grammar check failed');
    } finally {
      setGrammarLoading(false);
    }
  };

  const handleTranslate = async () => {
    const content = getDocContent?.() ?? '';
    if (!content.trim()) {
      setTranslateError('No document content to translate');
      return;
    }
    setTranslateLoading(true);
    setTranslateError('');
    setTranslateResult('');
    try {
      const res = await docsAI.translate(docId, content, translateLang);
      setTranslateResult(res.translated);
    } catch (e) {
      setTranslateError(e instanceof Error ? e.message : 'Translation failed');
    } finally {
      setTranslateLoading(false);
    }
  };

  const handleSummarize = async () => {
    const content = getDocContent?.() ?? '';
    if (!content.trim()) {
      setSummarizeError('No document content to summarize');
      return;
    }
    setSummarizeLoading(true);
    setSummarizeError('');
    setSummary('');
    try {
      const res = await docsAI.summarize(docId, content);
      setSummary(res.summary);
    } catch (e) {
      setSummarizeError(e instanceof Error ? e.message : 'Summarize failed');
    } finally {
      setSummarizeLoading(false);
    }
  };

  return (
    <div className={styles.panel}>
      <div className={styles.panelHeader}>
        <span className={styles.panelTitle}>
          <Sparkles size={15} />
          AI Assistant
        </span>
        <button className={styles.closeBtn} onClick={onClose} aria-label="Close AI panel">
          <X size={16} />
        </button>
      </div>

      {/* Help me write */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>
          <PenLine size={12} style={{ display: 'inline', marginRight: 4 }} />
          Help me write
        </div>
        <textarea
          className={styles.textarea}
          placeholder="Describe the document you want to create..."
          value={writeDesc}
          onChange={e => setWriteDesc(e.target.value)}
          rows={3}
        />
        <button
          className={`${styles.btn} ${styles.btnPrimary}`}
          onClick={handleHelpMeWrite}
          disabled={writeLoading || !writeDesc.trim()}
        >
          {writeLoading ? <span className={styles.spinner} /> : <Sparkles size={13} />}
          {writeLoading ? 'Generating...' : 'Generate'}
        </button>
        {writeError && <div className={styles.error}>{writeError}</div>}
        {writeResult && (
          <div className={styles.result}>
            {writeResult}
            {onInsertText && (
              <div className={styles.resultActions}>
                <button
                  className={styles.resultBtn}
                  onClick={() => onInsertText(writeResult)}
                >
                  Insert
                </button>
                <button
                  className={styles.resultBtn}
                  onClick={() => navigator.clipboard.writeText(writeResult)}
                >
                  Copy
                </button>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Grammar check */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>
          <SpellCheck size={12} style={{ display: 'inline', marginRight: 4 }} />
          Grammar &amp; Style
        </div>
        <button
          className={styles.btn}
          onClick={handleGrammarCheck}
          disabled={grammarLoading}
        >
          {grammarLoading ? <span className={styles.spinner} /> : <SpellCheck size={13} />}
          {grammarLoading ? 'Checking...' : 'Check Grammar'}
        </button>
        {grammarError && <div className={styles.error}>{grammarError}</div>}
        {grammarChecked && grammarIssues !== null && (
          grammarIssues.length === 0 ? (
            <div className={styles.noIssues}>No issues found!</div>
          ) : (
            <div className={styles.issueList}>
              {grammarIssues.map((issue, i) => (
                <div key={i} className={styles.issue}>
                  <div className={styles.issueMessage}>{issue.message}</div>
                  {issue.suggestion && (
                    <div className={styles.issueSuggestion}>
                      Suggestion: {issue.suggestion}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )
        )}
      </div>

      {/* Translate */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>
          <Languages size={12} style={{ display: 'inline', marginRight: 4 }} />
          Translate
        </div>
        <select
          className={styles.select}
          value={translateLang}
          onChange={e => setTranslateLang(e.target.value)}
        >
          {LANGUAGES.map(lang => (
            <option key={lang.code} value={lang.code}>{lang.label}</option>
          ))}
        </select>
        <button
          className={`${styles.btn} ${styles.btnPrimary}`}
          onClick={handleTranslate}
          disabled={translateLoading}
        >
          {translateLoading ? <span className={styles.spinner} /> : <Languages size={13} />}
          {translateLoading ? 'Translating...' : `Translate to ${translateLang}`}
        </button>
        {translateError && <div className={styles.error}>{translateError}</div>}
        {translateResult && (
          <div className={styles.result}>
            {translateResult}
            <div className={styles.resultActions}>
              <button
                className={styles.resultBtn}
                onClick={() => navigator.clipboard.writeText(translateResult)}
              >
                Copy
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Summarize */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>
          <BookOpen size={12} style={{ display: 'inline', marginRight: 4 }} />
          Summarize
        </div>
        <button
          className={styles.btn}
          onClick={handleSummarize}
          disabled={summarizeLoading}
        >
          {summarizeLoading ? <span className={styles.spinner} /> : <BookOpen size={13} />}
          {summarizeLoading ? 'Summarizing...' : 'Summarize Document'}
        </button>
        {summarizeError && <div className={styles.error}>{summarizeError}</div>}
        {summary && (
          <div className={styles.result}>
            {summary}
            <div className={styles.resultActions}>
              <button
                className={styles.resultBtn}
                onClick={() => navigator.clipboard.writeText(summary)}
              >
                Copy
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
