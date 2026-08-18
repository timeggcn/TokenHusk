import { useState, useCallback } from "react";
import type { DetectedApp, ConfigResult } from "../../types/ipc";
import { AppDetector } from "./AppDetector";
import { useAppDetection, useConfiguration } from "../../hooks/useConfig";

const PRESETS = [
  { id: "observe", name: "观测模式", desc: "只记录 Token 消耗，不压缩", savings: "0%" },
  { id: "conservative", name: "保守模式", desc: "仅清理结构噪声，不动语义", savings: "10-20%" },
  { id: "standard", name: "标准模式", desc: "推荐日常使用，平衡质量与节省", savings: "30-50%", recommended: true },
  { id: "aggressive", name: "激进模式", desc: "最大压缩，适合大量工具调用", savings: "50-70%" },
];

export function SetupWizard() {
  const [step, setStep] = useState(0);
  const [selectedApp, setSelectedApp] = useState<DetectedApp | null>(null);
  const [selectedPreset, setSelectedPreset] = useState("standard");
  const [result, setResult] = useState<ConfigResult | null>(null);
  const [executing, setExecuting] = useState(false);
  const { apps, loading, detect } = useAppDetection();
  const { result: configResult, configure, clearResult } = useConfiguration();

  const handleSelectApp = useCallback((app: DetectedApp) => {
    setSelectedApp(app);
    setStep(1);
  }, []);

  const handleConfirmTarget = useCallback(() => {
    setStep(2);
  }, []);

  const handleExecute = useCallback(async () => {
    if (!selectedApp) return;
    setExecuting(true);
    if (selectedApp.level === "A") {
      await configure(
        selectedApp.name,
        selectedApp.config_path,
        selectedApp.name === "ChatBox" ? "openaiBaseUrl" : "api.baseUrl",
        "http://127.0.0.1:10520",
        selectedApp.name === "ChatBox" ? "openaiKey" : "api.key"
      );
      setResult(configResult);
    }
    setStep(3);
    setExecuting(false);
  }, [selectedApp, configure, configResult]);

  const handleReset = useCallback(() => {
    setStep(0);
    setSelectedApp(null);
    setSelectedPreset("standard");
    setResult(null);
    clearResult();
    void detect();
  }, [detect, clearResult]);

  return (
    <div className="card">
      <div className="card-header">
        <h2 className="text-lg font-semibold text-ink">配置向导</h2>
        <p className="text-xs text-ink-muted mt-1">
          无需输入 API Key，TokenHusk 只修改 API 地址
        </p>
      </div>
      <div className="card-body">
        {/* 步骤指示器 */}
        <div className="flex items-center gap-2 mb-6">
          {["扫描应用", "确认目标", "选择预设", "执行配置"].map((label, i) => (
            <div key={label} className="flex items-center gap-2">
              <div
                className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-medium ${
                  i <= step ? "bg-primary text-white" : "bg-muted text-ink-muted"
                }`}
              >
                {i + 1}
              </div>
              <span className={`text-xs ${i <= step ? "text-ink" : "text-ink-muted"}`}>
                {label}
              </span>
              {i < 3 && <span className="text-line mx-1">—</span>}
            </div>
          ))}
        </div>

        {/* Step 0: 扫描应用 */}
        {step === 0 && (
          <AppDetector
            apps={apps}
            loading={loading}
            onScan={detect}
            onSelect={handleSelectApp}
          />
        )}

        {/* Step 1: 确认目标 */}
        {step === 1 && selectedApp && (
          <div className="space-y-4">
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
              <p className="text-sm text-blue-800">
                🔒 你的 API Key 保持在客户端中，TokenHusk 不会接触它
              </p>
            </div>
            <div className="border border-line rounded-lg p-4 space-y-3">
              <div className="flex justify-between">
                <span className="text-sm text-ink-muted">应用</span>
                <span className="text-sm text-ink font-medium">{selectedApp.name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-ink-muted">当前地址</span>
                <span className="text-sm font-mono">{selectedApp.current_base_url ?? "未设置"}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-ink-muted">新地址</span>
                <span className="text-sm font-mono text-primary">{selectedApp.suggested_base_url}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-ink-muted">API Key</span>
                <span className="text-sm text-success">
                  {selectedApp.api_key_present ? "✅ 已存在，保持不变" : "⚠️ 未检测到，请确保已配置"}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-ink-muted">安全级别</span>
                <span>
                  {selectedApp.level === "A" ? (
                    <span className="text-sm text-success">自动修改（已验证）</span>
                  ) : selectedApp.level === "B" ? (
                    <span className="text-sm text-amber-600">需确认修改</span>
                  ) : (
                    <span className="text-sm text-blue-600">手动配置（教程）</span>
                  )}
                </span>
              </div>
            </div>
            <div className="flex justify-end gap-3">
              <button type="button" onClick={() => setStep(0)} className="btn-secondary">
                返回
              </button>
              {selectedApp.level === "C" ? (
                <div className="border border-line rounded-lg p-4 bg-slate-50 max-w-md">
                  <p className="text-sm font-medium text-ink mb-2">手动配置步骤：</p>
                  <ol className="text-xs text-ink-muted space-y-1 list-decimal list-inside">
                    <li>打开 {selectedApp.name} 设置</li>
                    <li>将 API 地址改为：<code className="font-mono text-primary">{selectedApp.suggested_base_url}</code></li>
                    <li>API Key 保持不变</li>
                    <li>保存设置并测试连接</li>
                  </ol>
                  <button
                    type="button"
                    onClick={() => navigator.clipboard.writeText(selectedApp.suggested_base_url)}
                    className="mt-3 btn-ghost text-xs"
                  >
                    复制地址
                  </button>
                </div>
              ) : (
                <button type="button" onClick={handleConfirmTarget} className="btn-primary">
                  确认，下一步
                </button>
              )}
            </div>
          </div>
        )}

        {/* Step 2: 选择预设 */}
        {step === 2 && (
          <div className="space-y-4">
            <p className="text-sm text-ink-muted">
              选择压缩强度，可在设置中随时更改
            </p>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {PRESETS.map((preset) => (
                <button
                  key={preset.id}
                  type="button"
                  onClick={() => setSelectedPreset(preset.id)}
                  className={`text-left p-4 rounded-lg border transition-colors ${
                    selectedPreset === preset.id
                      ? "border-primary bg-blue-50"
                      : "border-line hover:border-primary/30"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-medium text-ink">
                      {preset.recommended && (
                        <span className="mr-1 text-xs text-accent">⭐</span>
                      )}
                      {preset.name}
                    </p>
                    <span className="text-xs font-mono text-primary">{preset.savings}</span>
                  </div>
                  <p className="text-xs text-ink-muted mt-1">{preset.desc}</p>
                </button>
              ))}
            </div>
            <div className="flex justify-end gap-3">
              <button type="button" onClick={() => setStep(1)} className="btn-secondary">
                返回
              </button>
              <button type="button" onClick={handleExecute} className="btn-primary" disabled={executing}>
                {executing ? "执行中…" : "执行配置"}
              </button>
            </div>
          </div>
        )}

        {/* Step 3: 完成 */}
        {step === 3 && (
          <div className="space-y-4">
            <div className={`rounded-lg p-4 ${result?.success ? "bg-green-50 border border-green-200" : "bg-red-50 border border-red-200"}`}>
              <p className={`text-sm font-medium ${result?.success ? "text-green-800" : "text-red-800"}`}>
                {result?.success ? "✅ 配置成功！" : "❌ 配置失败"}
              </p>
              <p className="text-xs mt-1 text-ink-muted">{result?.detail}</p>
            </div>

            {result?.success && (
              <div className="border border-line rounded-lg p-4 space-y-2">
                <p className="text-sm font-medium text-ink">配置摘要</p>
                <div className="text-xs text-ink-muted space-y-1">
                  <p>• 应用：{selectedApp?.name}</p>
                  <p>• 压缩预设：{PRESETS.find((p) => p.id === selectedPreset)?.name}</p>
                  <p>• 备份文件：{result?.backup_path}</p>
                  <p>• 连接测试：{result?.test_connection_ok ? "✅ 通过" : "⚠️ 未验证"}</p>
                </div>
              </div>
            )}

            <div className="flex justify-end gap-3">
              <button type="button" onClick={handleReset} className="btn-primary">
                完成
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}