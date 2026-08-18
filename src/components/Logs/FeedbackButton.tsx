import { useState } from "react";

async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    return null;
  }
}

interface Props {
  requestId: number;
}

export function FeedbackButton({ requestId }: Props) {
  const [feedback, setFeedback] = useState<"up" | "down" | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function submit(thumbsUp: boolean) {
    setSubmitting(true);
    await tryInvoke("submit_feedback", {
      request_id: requestId,
      thumbs_up: thumbsUp,
      comment: null as string | null,
    });
    setFeedback(thumbsUp ? "up" : "down");
    setSubmitting(false);
  }

  return (
    <div className="flex items-center gap-3">
      <p className="text-sm text-ink-muted">这次压缩质量如何？</p>
      <button
        type="button"
        disabled={submitting || feedback !== null}
        onClick={() => submit(true)}
        className={`btn min-h-[36px] px-3 text-sm ${
          feedback === "up" ? "bg-success text-white" : "btn-secondary"
        }`}
      >
        👍 很好
      </button>
      <button
        type="button"
        disabled={submitting || feedback !== null}
        onClick={() => submit(false)}
        className={`btn min-h-[36px] px-3 text-sm ${
          feedback === "down" ? "bg-destructive text-white" : "btn-secondary"
        }`}
      >
        👎 变差了
      </button>
      {feedback && (
        <span className="text-xs text-ink-muted">已提交反馈</span>
      )}
    </div>
  );
}