import { useState } from "react";
import { useEmergencyRestore } from "../../hooks/useConfig";
import type { ConfigResult } from "../../types/ipc";

export function EmergencyRestore() {
  const [showConfirm, setShowConfirm] = useState(false);
  const { restoring, results, restore, clearResults } = useEmergencyRestore();

  function handleRestore() {
    setShowConfirm(true);
  }

  function confirmRestore() {
    void restore();
    setShowConfirm(false);
  }

  function statusIcon(r: ConfigResult) {
    return r.success ? "✅" : "❌";
  }

  return (
    <div className="card">
      <div className="card-header">
        <h3 className="text-sm font-semibold text-ink">紧急还原</h3>
        <p className="text-xs text-ink-muted mt-1">
          一键恢复所有客户端的原始 API 地址（从 .bak 备份恢复）
        </p>
      </div>
      <div className="card-body space-y-4">
        <div className="bg-amber-50 border border-amber-200 rounded-lg p-4">
          <p className="text-sm text-amber-800">
            ⚠️ 此操作将恢复所有被 TokenHusk 修改过的配置文件到原始状态。
            还原后 AI 客户端将直接连接到原始 API 地址，不再经过 TokenHusk 代理。
          </p>
          <p className="text-xs text-amber-700 mt-2">
            ❗ 此操作不依赖 TokenHusk 代理运行状态
          </p>
        </div>

        {!showConfirm && results.length === 0 && (
          <button
            type="button"
            onClick={handleRestore}
            className="btn-destructive w-full"
            disabled={restoring}
          >
            {restoring ? "还原中…" : "一键紧急还原"}
          </button>
        )}

        {showConfirm && (
          <div className="border border-red-200 rounded-lg p-4 bg-red-50 space-y-3">
            <p className="text-sm font-medium text-red-800">
              确定要执行紧急还原吗？
            </p>
            <div className="flex gap-3">
              <button
                type="button"
                onClick={confirmRestore}
                className="btn-destructive"
              >
                确认还原
              </button>
              <button
                type="button"
                onClick={() => setShowConfirm(false)}
                className="btn-secondary"
              >
                取消
              </button>
            </div>
          </div>
        )}

        {results.length > 0 && (
          <div className="space-y-2">
            <p className="text-sm font-medium text-ink">还原结果</p>
            <div className="divide-y divide-line border border-line rounded-lg">
              {results.map((r, i) => (
                <div key={i} className="flex items-center justify-between px-4 py-3">
                  <div className="flex items-center gap-3">
                    <span>{statusIcon(r)}</span>
                    <div>
                      <p className="text-sm text-ink">{r.app_name}</p>
                      <p className="text-xs text-ink-muted">{r.detail}</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
            <button
              type="button"
              onClick={clearResults}
              className="btn-ghost text-sm"
            >
              清除结果
            </button>
          </div>
        )}
      </div>
    </div>
  );
}