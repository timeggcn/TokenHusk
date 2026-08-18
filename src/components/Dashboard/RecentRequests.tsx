import { useState, useCallback } from "react";
import type { RequestRecord } from "../../types/ipc";

async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    return null;
  }
}

function mockRequests(): RequestRecord[] {
  return Array.from({ length: 8 }, (_, i) => ({
    id: i + 1,
    timestamp: new Date(Date.now() - i * 60000).toISOString(),
    source_app: ["Claude Code", "Cursor", "ChatBox", "Cherry Studio"][i % 4],
    provider: ["anthropic", "openai", "deepseek"][i % 3],
    model: ["claude-sonnet-4", "gpt-4o", "deepseek-chat"][i % 3],
    original_input_tokens: 42000 - i * 1000,
    compressed_input_tokens: 18000 - i * 500,
    output_tokens: 500,
    saved_tokens: 24000 - i * 500,
    saved_ratio: 0.57 + i * 0.01,
    estimated_cost_usd: 0.15,
    saved_cost_usd: 0.08,
    stages_applied: ["JsonCrusher", "LogDedup"],
    compression_time_ms: 12,
    skipped: i === 3,
    skip_reason: i === 3 ? "压缩后 token ≥ 原始" : null,
    message_count: 24,
    has_code: i % 2 === 0,
    has_json: true,
    has_log: i % 3 === 0,
    original_body: JSON.stringify({ messages: [{ role: "user", content: "hello" }] }),
    compressed_body: JSON.stringify({ messages: [{ role: "user", content: "hello" }] }),
  }));
}

interface Props {
  onSelectRequest: (id: number) => void;
}

export function RecentRequests({ onSelectRequest }: Props) {
  const [requests, setRequests] = useState<RequestRecord[]>(mockRequests);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    const data = await tryInvoke<RequestRecord[]>("get_recent_requests", { limit: 50 });
    if (data) setRequests(data);
    setLoading(false);
  }, []);

  function formatTime(ts: string): string {
    const d = new Date(ts);
    return d.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
  }

  function ratioLabel(r: RequestRecord): { text: string; cls: string } {
    if (r.skipped) return { text: "跳过", cls: "badge-warning" };
    const pct = (r.saved_ratio * 100).toFixed(0);
    if (r.saved_ratio > 0.5) return { text: `-${pct}%`, cls: "badge-success" };
    if (r.saved_ratio > 0.2) return { text: `-${pct}%`, cls: "badge-info" };
    return { text: `-${pct}%`, cls: "badge-warning" };
  }

  return (
    <div className="card">
      <div className="card-header flex items-center justify-between">
        <h3 className="text-sm font-semibold text-ink">最近请求</h3>
        <button
          type="button"
          onClick={refresh}
          className="btn-ghost text-xs"
          disabled={loading}
        >
          {loading ? "刷新中…" : "刷新"}
        </button>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-line text-ink-subtle text-xs uppercase tracking-wider">
              <th className="px-4 py-3 text-left font-medium">时间</th>
              <th className="px-4 py-3 text-left font-medium">来源</th>
              <th className="px-4 py-3 text-left font-medium">服务商</th>
              <th className="px-4 py-3 text-right font-medium">原始</th>
              <th className="px-4 py-3 text-right font-medium">压缩后</th>
              <th className="px-4 py-3 text-right font-medium">压缩率</th>
              <th className="px-4 py-3 text-center font-medium">状态</th>
            </tr>
          </thead>
          <tbody>
            {requests.map((req) => {
              const ratio = ratioLabel(req);
              return (
                <tr
                  key={req.id}
                  className="border-b border-line hover:bg-slate-50 cursor-pointer transition-colors"
                  onClick={() => onSelectRequest(req.id)}
                >
                  <td className="px-4 py-3 text-ink-muted font-mono text-xs">
                    {formatTime(req.timestamp)}
                  </td>
                  <td className="px-4 py-3 text-ink">{req.source_app}</td>
                  <td className="px-4 py-3 text-ink-muted">{req.provider}</td>
                  <td className="px-4 py-3 text-right font-mono text-xs">
                    {(req.original_input_tokens / 1000).toFixed(1)}K
                  </td>
                  <td className="px-4 py-3 text-right font-mono text-xs">
                    {(req.compressed_input_tokens / 1000).toFixed(1)}K
                  </td>
                  <td className="px-4 py-3 text-right">
                    <span className={`${ratio.cls} text-xs`}>{ratio.text}</span>
                  </td>
                  <td className="px-4 py-3 text-center">
                    {req.skipped ? (
                      <span className="badge-warning text-xs">跳过</span>
                    ) : (
                      <span className="badge-success text-xs">已压缩</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}