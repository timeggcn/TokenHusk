import { useCallback, useEffect, useRef, useState } from "react";
import type { DashboardOverview } from "../types/ipc";

// 尝试调用 Tauri IPC，不可用时回落 mock 数据
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    return null;
  }
}

function mockOverview(): DashboardOverview {
  return {
    today_requests: 147,
    today_saved_tokens: 892340,
    today_saved_ratio: 0.42,
    today_saved_cost: 8.94,
    today_estimated_cost: 21.28,
    total_requests_all_time: 12531,
    proxy_running: true,
    proxy_uptime_seconds: 3720,
    upstream: "https://api.openai.com",
  };
}

export function useStats() {
  const [overview, setOverview] = useState<DashboardOverview>(mockOverview());
  const [loading, setLoading] = useState(false);
  const intervalRef = useRef<ReturnType<typeof setInterval>>();

  const fetchStats = useCallback(async () => {
    setLoading(true);
    const data = await tryInvoke<DashboardOverview>("get_stats");
    if (data) {
      setOverview(data);
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    void fetchStats();
    intervalRef.current = setInterval(fetchStats, 5000);
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [fetchStats]);

  return { overview, loading, refresh: fetchStats };
}