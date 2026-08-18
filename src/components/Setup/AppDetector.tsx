import type { DetectedApp } from "../../types/ipc";

interface Props {
  apps: DetectedApp[];
  loading: boolean;
  onScan: () => void;
  onSelect: (app: DetectedApp) => void;
}

export function AppDetector({ apps, loading, onScan, onSelect }: Props) {
  function levelBadge(level: string) {
    switch (level) {
      case "A": return <span className="badge-success">A 级 · 自动</span>;
      case "B": return <span className="badge-warning">B 级 · 需确认</span>;
      case "C": return <span className="badge-info">C 级 · 教程</span>;
      default: return null;
    }
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-semibold text-ink">扫描已安装的 AI 应用</h3>
          <p className="text-xs text-ink-muted mt-1">
            TokenHusk 会自动检测你电脑上已安装的 AI 客户端
          </p>
        </div>
        <button
          type="button"
          onClick={onScan}
          className="btn-primary"
          disabled={loading}
        >
          {loading ? "扫描中…" : "开始扫描"}
        </button>
      </div>

      {apps.length > 0 && (
        <div className="divide-y divide-line border border-line rounded-lg">
          {apps.map((app) => (
            <button
              key={app.name}
              type="button"
              onClick={() => onSelect(app)}
              className="w-full flex items-center justify-between px-4 py-3 hover:bg-slate-50 transition-colors text-left"
            >
              <div className="flex items-center gap-3">
                <div>
                  <p className="text-sm font-medium text-ink">{app.name}</p>
                  <p className="text-xs text-ink-muted font-mono">
                    {app.config_path}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {app.configured && (
                  <span className="badge-success">已配置</span>
                )}
                {levelBadge(app.level)}
                {app.api_key_present && !app.configured && (
                  <span className="text-xs text-ink-muted">Key 已存在</span>
                )}
              </div>
            </button>
          ))}
        </div>
      )}

      {!loading && apps.length === 0 && (
        <div className="text-center py-8 text-ink-muted text-sm">
          请点击"开始扫描"检测已安装的应用
        </div>
      )}
    </div>
  );
}