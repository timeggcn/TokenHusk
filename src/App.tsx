import { useState, useCallback } from "react";
import type { TabType } from "./types/ipc";
import { useStats } from "./hooks/useStats";
import { Overview } from "./components/Dashboard/Overview";
import { TokenDistributionChart } from "./components/Dashboard/TokenDistribution";
import { SavingsChart } from "./components/Dashboard/SavingsChart";
import { RecentRequests } from "./components/Dashboard/RecentRequests";
import { RequestDetail } from "./components/Logs/RequestDetail";
import { SetupWizard } from "./components/Setup/SetupWizard";
import { EmergencyRestore } from "./components/Safety/EmergencyRestore";
import { PauseToggle } from "./components/Safety/PauseToggle";

const TABS: { id: TabType; label: string; icon: string }[] = [
  { id: "dashboard", label: "仪表盘", icon: "📊" },
  { id: "logs", label: "请求日志", icon: "📋" },
  { id: "setup", label: "配置向导", icon: "⚙" },
  { id: "safety", label: "安全兜底", icon: "🛡" },
];

export function App(): JSX.Element {
  const [currentTab, setCurrentTab] = useState<TabType>("dashboard");
  const [selectedRequestId, setSelectedRequestId] = useState<number | null>(null);
  const [proxyPaused, setProxyPaused] = useState(false);
  const { overview } = useStats();

  const handleSelectRequest = useCallback((id: number) => {
    setSelectedRequestId(id);
  }, []);

  const handleCloseRequestDetail = useCallback(() => {
    setSelectedRequestId(null);
  }, []);

  const handleToggleProxy = useCallback(async (paused: boolean) => {
    setProxyPaused(paused);
    // 通过 IPC 通知后端
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      if (paused) {
        await invoke("pause_proxy");
      } else {
        await invoke("resume_proxy");
      }
    } catch {
      // 开发模式不可用，忽略
    }
  }, []);

  return (
    <div className="min-h-screen flex">
      {/* 侧边栏 */}
      <aside className="w-56 bg-white border-r border-line flex flex-col">
        <div className="px-5 py-5 border-b border-line">
          <h1 className="text-lg font-bold text-primary">TokenHusk</h1>
          <p className="text-xs text-ink-muted mt-0.5">AI Agent 成本控制层</p>
        </div>
        <nav className="flex-1 py-3">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => setCurrentTab(tab.id)}
              className={`w-full flex items-center gap-3 px-5 py-3 text-sm transition-colors ${
                currentTab === tab.id
                  ? "bg-primary/5 text-primary font-medium border-r-2 border-primary"
                  : "text-ink-muted hover:bg-slate-50"
              }`}
            >
              <span className="text-base">{tab.icon}</span>
              {tab.label}
            </button>
          ))}
        </nav>
        <div className="px-5 py-4 border-t border-line">
          <div className="flex items-center gap-2">
            <div
              className={`w-2 h-2 rounded-full ${
                proxyPaused ? "bg-amber-500" : overview.proxy_running ? "bg-success" : "bg-destructive"
              }`}
            />
            <span className="text-xs text-ink-muted">
              {proxyPaused ? "已暂停" : overview.proxy_running ? "运行中" : "离线"}
            </span>
          </div>
          <p className="text-xs text-ink-subtle mt-1 font-mono">
            127.0.0.1:10520
          </p>
        </div>
      </aside>

      {/* 主内容 */}
      <main className="flex-1 overflow-auto">
        <div className="p-6 max-w-6xl mx-auto">
          {/* 仪表盘 */}
          {currentTab === "dashboard" && (
            <div className="space-y-6">
              <div className="flex items-center justify-between">
                <h2 className="text-xl font-semibold text-ink">仪表盘</h2>
                <span className="text-xs text-ink-muted">
                  上游：{overview.upstream}
                </span>
              </div>
              <Overview overview={overview} />
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                <TokenDistributionChart />
                <SavingsChart />
              </div>
              <RecentRequests onSelectRequest={handleSelectRequest} />
            </div>
          )}

          {/* 请求日志 */}
          {currentTab === "logs" && (
            <div className="space-y-6">
              <h2 className="text-xl font-semibold text-ink">请求日志</h2>
              <RecentRequests onSelectRequest={handleSelectRequest} />
            </div>
          )}

          {/* 配置向导 */}
          {currentTab === "setup" && (
            <div className="space-y-6">
              <h2 className="text-xl font-semibold text-ink">配置向导</h2>
              <SetupWizard />
            </div>
          )}

          {/* 安全兜底 */}
          {currentTab === "safety" && (
            <div className="space-y-6">
              <h2 className="text-xl font-semibold text-ink">安全兜底</h2>
              <PauseToggle paused={proxyPaused} onToggle={handleToggleProxy} />
              <EmergencyRestore />
            </div>
          )}
        </div>
      </main>

      {/* 请求详情弹窗 */}
      {selectedRequestId && (
        <RequestDetail
          requestId={selectedRequestId}
          onClose={handleCloseRequestDetail}
        />
      )}
    </div>
  );
}