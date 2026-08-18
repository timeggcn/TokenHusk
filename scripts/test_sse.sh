#!/usr/bin/env bash
# TokenHusk Phase 0 端到端验证：本地 mock LLM + 本地代理 + curl 断言。
#
# 前置：
#   1. Rust 工具链可编译（见 scripts/env.sh + .cargo/config.toml）
#   2. Node >= 18（scripts/env.sh 已注入）
#   3. 已构建：source scripts/env.sh && cargo build -p tokenhusk-core
#
# 用法：bash scripts/test_sse.sh
#
# 说明（AGENTS.md Rule 4）：mock 只回显 Authorization 的脱敏占位，
#   任何环节不落盘 / 打印真实 Key。

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROXY_URL="http://127.0.0.1:10520"
MOCK_PORT=3001
MOCK_URL="http://127.0.0.1:${MOCK_PORT}"

# 工具链（本机 FlyEnv 路径）
if [ -f "$ROOT/scripts/env.sh" ]; then
  # shellcheck disable=SC1090
  source "$ROOT/scripts/env.sh"
fi

cd "$ROOT"

MOCK_PID=""
PROXY_PID=""
cleanup() {
  echo "--- cleanup: killing mock($MOCK_PID) proxy($PROXY_PID) ---"
  [ -n "$PROXY_PID" ] && kill "$PROXY_PID" 2>/dev/null || true
  [ -n "$MOCK_PID" ] && kill "$MOCK_PID" 2>/dev/null || true
  exit "${1:-0}"
}
trap 'cleanup' EXIT

echo "=== 1. 构建 phase0_poc ==="
cargo build -p tokenhusk-core --bin phase0_poc --quiet

echo "=== 2. 启动 mock LLM server (port $MOCK_PORT) ==="
node scripts/mock_llm_server.mjs &
MOCK_PID=$!

echo "=== 3. 启动代理（上游 = mock）==="
TOKENHUSK_UPSTREAM="$MOCK_URL" ./target/debug/phase0_poc &
PROXY_PID=$!

echo "=== 4. 等待 /health 就绪 ==="
READY=0
for _ in $(seq 1 30); do
  if curl -sS -o /dev/null -w "%{http_code}" "$PROXY_URL/health" 2>/dev/null | grep -q "200"; then
    READY=1
    break
  fi
  sleep 1
done
if [ "$READY" -ne 1 ]; then
  echo "FAIL: /health 未在 30s 内就绪"
  cleanup 1
fi
echo "PASS: /health -> 200"

echo "=== 5. 流式 SSE 透传断言 ==="
STREAM_OUT="$(curl -sS -N -X POST "$PROXY_URL/v1/chat/completions" \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer sk-mock-REDACTED-ONLY' \
  -d '{"model":"mock-gpt","stream":true,"messages":[{"role":"user","content":"hi"}]}')"

echo "$STREAM_OUT" | sed -n '1,12p'

echo "$STREAM_OUT" | grep -q "data: \[DONE\]" || { echo "FAIL: 未收到 [DONE]"; cleanup 1; }
DATA_LINES=$(echo "$STREAM_OUT" | grep -c "^data: ")
if [ "$DATA_LINES" -lt 3 ]; then
  echo "FAIL: 收到的 data: 行过少 ($DATA_LINES)，疑似未流式透传"
  cleanup 1
fi
echo "PASS: SSE 流（$DATA_LINES 个 data: 行 + [DONE]）"

echo "=== 6. 非流式 JSON 透传断言 ==="
JSON_OUT="$(curl -sS -X POST "$PROXY_URL/v1/chat/completions" \
  -H 'Content-Type: application/json' \
  -d '{"model":"mock-gpt","stream":false,"messages":[{"role":"user","content":"hi"}]}')"
echo "$JSON_OUT" | grep -q '"object":"chat.completion"' || { echo "FAIL: 非流式响应异常"; cleanup 1; }
echo "PASS: 非流式 JSON"

echo ""
echo "=============================================="
echo "  ALL CHECKS PASSED"
echo "=============================================="
cleanup 0
