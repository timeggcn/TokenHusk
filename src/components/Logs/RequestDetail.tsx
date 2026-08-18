import { useState, useEffect, useCallback } from "react";
import type { RequestRecord } from "../../types/ipc";
import { DiffViewer } from "./DiffViewer";
import { FeedbackButton } from "./FeedbackButton";

async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    return null;
  }
}

interface Props {
  requestId: number | null;
  onClose: () => void;
}

function formatDate(ts: string): string {
  const d = new Date(ts);
  return d.toLocaleString("zh-CN");
}

export function RequestDetail({ requestId, onClose }: Props) {
  const [record, setRecord] = useState<RequestRecord | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchDetail = useCallback(async () => {
    if (!requestId) return;
    setLoading(true);
    const data = await tryInvoke<RequestRecord>("get_request_detail", { request_id: requestId });
    if (data) setRecord(data);
    setLoading(false);
  }, [requestId]);

  useEffect(() => {
    if (requestId) void fetchDetail();
  }, [requestId, fetchDetail]);

  if (!requestId) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/30 flex items-start justify-center pt-12 overflow-auto">
      <div className="w-full max-w-5xl mx-4 mb-12 bg-white rounded-xl shadow-xl border border-line">
        <div className="flex items-center justify-between px-6 py-4 border-b border-line">
          <h2 className="text-lg font-semibold text-ink">
            请求详情
            {record && <span className="ml-2 text-sm font-normal text-ink-muted">#{record.id}</span>}
          </h2>
          <button type="button" onClick={onClose} className="btn-ghost">
            关闭
          </button>
        </div>

        {loading && (
          <div className="p-12 text-center text-ink-muted">加载中…</div>
        )}

        {record && (
          <div className="p-6 space-y-6">
            {/* 元信息 */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div>
                <p className="text-xs text-ink-subtle">时间</p>
                <p className="text-sm text-ink">{formatDate(record.timestamp)}</p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">来源</p>
                <p className="text-sm text-ink">{record.source_app}</p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">服务商</p>
                <p className="text-sm text-ink">{record.provider}</p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">模型</p>
                <p className="text-sm text-ink">{record.model}</p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">原始 Token</p>
                <p className="text-sm font-mono">{record.original_input_tokens}</p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">压缩后 Token</p>
                <p className="text-sm font-mono">{record.compressed_input_tokens}</p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">节省</p>
                <p className="text-sm font-mono text-success">
                  {record.saved_tokens} ({(record.saved_ratio * 100).toFixed(1)}%)
                </p>
              </div>
              <div>
                <p className="text-xs text-ink-subtle">压缩耗时</p>
                <p className="text-sm font-mono">{record.compression_time_ms}ms</p>
              </div>
            </div>

            {/* 应用 Stage */}
            <div>
              <p className="text-xs text-ink-subtle mb-1">应用阶段</p>
              <div className="flex gap-2 flex-wrap">
                {record.stages_applied.map((stage) => (
                  <span key={stage} className="badge-info">{stage}</span>
                ))}
                {record.skipped && (
                  <span className="badge-warning">
                    跳过{record.skip_reason ? `: ${record.skip_reason}` : ""}
                  </span>
                )}
              </div>
            </div>

            {/* Diff 查看器 */}
            <DiffViewer record={record} />

            {/* 反馈 */}
            <FeedbackButton requestId={record.id} />
          </div>
        )}
      </div>
    </div>
  );
}