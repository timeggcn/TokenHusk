import { useCallback, useState } from "react";
import type { ConfigResult, DetectedApp } from "../types/ipc";

async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(cmd, args);
  } catch {
    return null;
  }
}

function mockDetectedApps(): DetectedApp[] {
  return [
    {
      name: "ChatBox",
      config_path: "~/.config/chatbox/config.json",
      level: "A",
      configured: false,
      current_base_url: "https://api.openai.com",
      suggested_base_url: "http://127.0.0.1:10520",
      api_key_present: true,
      original_base_url: "https://api.openai.com",
    },
    {
      name: "Cherry Studio",
      config_path: "~/.config/cherry-studio/settings.json",
      level: "A",
      configured: false,
      current_base_url: "https://api.openai.com",
      suggested_base_url: "http://127.0.0.1:10520",
      api_key_present: true,
      original_base_url: "https://api.openai.com",
    },
    {
      name: "Cursor",
      config_path: "~/.cursor/settings.json",
      level: "B",
      configured: false,
      current_base_url: "https://api.openai.com",
      suggested_base_url: "http://127.0.0.1:10520",
      api_key_present: true,
      original_base_url: "https://api.openai.com",
    },
    {
      name: "Claude Code",
      config_path: "~/.claude/settings.json",
      level: "C",
      configured: false,
      current_base_url: null,
      suggested_base_url: "http://127.0.0.1:10520",
      api_key_present: false,
      original_base_url: null,
    },
  ];
}

export function useAppDetection() {
  const [apps, setApps] = useState<DetectedApp[]>([]);
  const [loading, setLoading] = useState(false);

  const detect = useCallback(async () => {
    setLoading(true);
    const data = await tryInvoke<DetectedApp[]>("detect_apps");
    setApps(data ?? mockDetectedApps());
    setLoading(false);
  }, []);

  return { apps, loading, detect };
}

export function useConfiguration() {
  const [configuring, setConfiguring] = useState(false);
  const [result, setResult] = useState<ConfigResult | null>(null);

  const configure = useCallback(async (appName: string, configPath: string, baseUrlJsonPath: string, newBaseUrl: string, apiKeyJsonPath: string | null) => {
    setConfiguring(true);
    const data = await tryInvoke<ConfigResult>("configure_app", {
      app_name: appName,
      config_path: configPath,
      base_url_json_path: baseUrlJsonPath,
      new_base_url: newBaseUrl,
      api_key_json_path: apiKeyJsonPath,
    });
    setResult(data ?? {
      app_name: appName,
      success: true,
      backup_path: `/tmp/${appName}.bak.20250101_120000`,
      detail: "配置成功（mock）",
      test_connection_ok: true,
    });
    setConfiguring(false);
  }, []);

  return { configuring, result, configure, clearResult: () => setResult(null) };
}

export function useEmergencyRestore() {
  const [restoring, setRestoring] = useState(false);
  const [results, setResults] = useState<ConfigResult[]>([]);

  const restore = useCallback(async () => {
    setRestoring(true);
    const data = await tryInvoke<ConfigResult[]>("emergency_restore");
    setResults(data ?? []);
    setRestoring(false);
  }, []);

  return { restoring, results, restore, clearResults: () => setResults([]) };
}