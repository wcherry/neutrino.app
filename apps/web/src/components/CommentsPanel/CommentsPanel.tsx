'use client';

import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { commentsApi, type Comment, type CommentReply } from '@/lib/api';
import { MessageSquare, X, Check, Trash2, Reply, ChevronDown, ChevronRight } from 'lucide-react';
import styles from './CommentsPanel.module.css';

interface CommentsPanelProps {
  fileId: string;
  onClose: () => void;
  /** Pre-fill the new-comment textarea (e.g. from a context-menu "Add comment" action) */
  initialText?: string;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

// ── Reply form ────────────────────────────────────────────────────────────────

function ReplyForm({
  fileId,
  commentId,
  onDone,
}: {
  fileId: string;
  commentId: string;
  onDone: () => void;
}) {
  const queryClient = useQueryClient();
  const [text, setText] = useState('');

  const mutation = useMutation({
    mutationFn: (body: string) => commentsApi.addReply(fileId, commentId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['comments', fileId] });
      setText('');
      onDone();
    },
  });

  function submit() {
    const body = text.trim();
    if (!body || mutation.isPending) return;
    mutation.mutate(body);
  }

  return (
    <div className={styles.replyForm}>
      <textarea
        className={styles.textarea}
        placeholder="Reply…"
        value={text}
        onChange={e => setText(e.target.value)}
        rows={2}
        onKeyDown={e => {
          if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) submit();
          if (e.key === 'Escape') onDone();
        }}
        autoFocus
      />
      <div className={styles.formActions}>
        <button className={styles.cancelBtn} onClick={onDone}>Cancel</button>
        <button
          className={styles.submitBtn}
          onClick={submit}
          disabled={!text.trim() || mutation.isPending}
        >
          {mutation.isPending ? 'Replying…' : 'Reply'}
        </button>
      </div>
    </div>
  );
}

// ── Single reply ──────────────────────────────────────────────────────────────

function ReplyRow({
  reply,
  fileId,
  commentId,
}: {
  reply: CommentReply;
  fileId: string;
  commentId: string;
}) {
  const queryClient = useQueryClient();

  const deleteMutation = useMutation({
    mutationFn: () => commentsApi.deleteReply(fileId, commentId, reply.id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['comments', fileId] }),
  });

  return (
    <div className={styles.reply}>
      <div className={styles.replyHeader}>
        <span className={styles.avatar}>{reply.userName.charAt(0).toUpperCase()}</span>
        <span className={styles.authorName}>{reply.userName}</span>
        <span className={styles.timestamp}>{formatDate(reply.createdAt)}</span>
        <button
          className={styles.iconBtn}
          title="Delete reply"
          onClick={() => deleteMutation.mutate()}
          disabled={deleteMutation.isPending}
        >
          <Trash2 size={11} />
        </button>
      </div>
      <p className={styles.replyBody}>{reply.body}</p>
    </div>
  );
}

// ── Single comment thread ─────────────────────────────────────────────────────

function CommentThread({ comment, fileId }: { comment: Comment; fileId: string }) {
  const queryClient = useQueryClient();
  const [showReplies, setShowReplies] = useState(true);
  const [showReplyForm, setShowReplyForm] = useState(false);

  const resolveMutation = useMutation({
    mutationFn: () =>
      commentsApi.updateComment(fileId, comment.id, {
        status: comment.status === 'open' ? 'resolved' : 'open',
      }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['comments', fileId] }),
  });

  const deleteMutation = useMutation({
    mutationFn: () => commentsApi.deleteComment(fileId, comment.id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['comments', fileId] }),
  });

  const isResolved = comment.status === 'resolved';

  return (
    <div className={`${styles.thread} ${isResolved ? styles.threadResolved : ''}`}>
      {/* Thread header */}
      <div className={styles.threadHeader}>
        <span className={styles.avatar}>{comment.userName.charAt(0).toUpperCase()}</span>
        <div className={styles.threadMeta}>
          <span className={styles.authorName}>{comment.userName}</span>
          <span className={styles.timestamp}>{formatDate(comment.createdAt)}</span>
        </div>
        <div className={styles.threadActions}>
          <button
            className={styles.iconBtn}
            title={isResolved ? 'Reopen' : 'Resolve'}
            onClick={() => resolveMutation.mutate()}
            disabled={resolveMutation.isPending}
          >
            <Check size={13} />
          </button>
          <button
            className={styles.iconBtn}
            title="Delete"
            onClick={() => deleteMutation.mutate()}
            disabled={deleteMutation.isPending}
          >
            <Trash2 size={13} />
          </button>
        </div>
      </div>

      {/* Comment body */}
      <p className={styles.commentBody}>{comment.body}</p>

      {/* Status badge */}
      {isResolved && (
        <span className={styles.resolvedBadge}>Resolved</span>
      )}

      {/* Replies */}
      {comment.replies.length > 0 && (
        <button
          className={styles.repliesToggle}
          onClick={() => setShowReplies(v => !v)}
        >
          {showReplies ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
          {comment.replies.length} {comment.replies.length === 1 ? 'reply' : 'replies'}
        </button>
      )}

      {showReplies && comment.replies.map(r => (
        <ReplyRow key={r.id} reply={r} fileId={fileId} commentId={comment.id} />
      ))}

      {/* Reply toggle */}
      {!showReplyForm && !isResolved && (
        <button
          className={styles.replyBtn}
          onClick={() => setShowReplyForm(true)}
        >
          <Reply size={11} /> Reply
        </button>
      )}

      {showReplyForm && (
        <ReplyForm
          fileId={fileId}
          commentId={comment.id}
          onDone={() => setShowReplyForm(false)}
        />
      )}
    </div>
  );
}

// ── Main panel ────────────────────────────────────────────────────────────────

export function CommentsPanel({ fileId, onClose, initialText }: CommentsPanelProps) {
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<'open' | 'all'>('open');
  const [newText, setNewText] = useState(initialText ?? '');
  const newTextareaRef = React.useRef<HTMLTextAreaElement>(null);

  // Focus and scroll to the compose box when opened with pre-filled text
  React.useEffect(() => {
    if (initialText) {
      newTextareaRef.current?.focus();
    }
  }, [initialText]);

  const { data, isLoading, isError } = useQuery({
    queryKey: ['comments', fileId, filter],
    queryFn: () => commentsApi.listComments(fileId, filter === 'open' ? 'open' : undefined),
    staleTime: 10_000,
    refetchInterval: 30_000,
  });

  const createMutation = useMutation({
    mutationFn: (body: string) => commentsApi.createComment(fileId, body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['comments', fileId] });
      setNewText('');
    },
  });

  function submitNew() {
    const body = newText.trim();
    if (!body || createMutation.isPending) return;
    createMutation.mutate(body);
  }

  const comments = data?.comments ?? [];
  const openCount = comments.filter(c => c.status === 'open').length;

  return (
    <div className={styles.panel}>
      {/* Header */}
      <div className={styles.header}>
        <div className={styles.headerTitle}>
          <MessageSquare size={16} />
          Comments
          {openCount > 0 && <span className={styles.countBadge}>{openCount}</span>}
        </div>
        <div className={styles.headerRight}>
          <select
            className={styles.filterSelect}
            value={filter}
            onChange={e => setFilter(e.target.value as 'open' | 'all')}
          >
            <option value="open">Open</option>
            <option value="all">All</option>
          </select>
          <button className={styles.closeBtn} onClick={onClose} title="Close">
            <X size={16} />
          </button>
        </div>
      </div>

      {/* New comment form */}
      <div className={styles.newCommentForm}>
        <textarea
          ref={newTextareaRef}
          className={styles.textarea}
          placeholder="Add a comment… (use @name to mention)"
          value={newText}
          onChange={e => setNewText(e.target.value)}
          rows={3}
          onKeyDown={e => {
            if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) submitNew();
          }}
        />
        <button
          className={styles.submitBtn}
          onClick={submitNew}
          disabled={!newText.trim() || createMutation.isPending}
        >
          {createMutation.isPending ? 'Posting…' : 'Comment'}
        </button>
      </div>

      {/* Thread list */}
      <div className={styles.list}>
        {isLoading && <div className={styles.empty}>Loading comments…</div>}
        {isError && (
          <div className={styles.empty} style={{ color: 'var(--color-danger, #dc2626)' }}>
            Failed to load comments.
          </div>
        )}
        {!isLoading && !isError && comments.length === 0 && (
          <div className={styles.empty}>
            {filter === 'open' ? 'No open comments.' : 'No comments yet.'}
          </div>
        )}
        {comments.map(c => (
          <CommentThread key={c.id} comment={c} fileId={fileId} />
        ))}
      </div>
    </div>
  );
}
