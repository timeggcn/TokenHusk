// TokenHusk Phase 0 端到端验证用 Mock LLM Server。
//
// 用途：离线验证代理的 SSE 流式透传（Node 18+，零依赖，ESM）。
// 模拟 OpenAI Chat Completions 的两种响应：
//   stream:true  -> SSE 逐块下发（50ms 间隔，可验证代理无缓冲）
//   stream:false -> 一次性 JSON
//
// 启动：
//   node scripts/mock_llm_server.mjs            # 默认 127.0.0.1:3001
//   MOCK_PORT=4000 node scripts/mock_llm_server.mjs

import http from "node:http";

const PORT = Number(process.env.MOCK_PORT ?? 3001);
const HOST = "127.0.0.1";

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function sseChunk(content, model, done = false) {
  return {
    id: "chatcmpl-mock",
    object: "chat.completion.chunk",
    created: 1_700_000_000,
    model,
    choices: [
      {
        index: 0,
        delta: done ? {} : { content },
        finish_reason: done ? "stop" : null,
      },
    ],
  };
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let data = "";
    req.on("data", (c) => (data += c));
    req.on("end", () => resolve(data));
    req.on("error", reject);
  });
}

const server = http.createServer(async (req, res) => {
  const url = req.url || "/";
  const auth = req.headers["authorization"] || "(none)";
  console.log(`[mock] ${req.method} ${url}  Authorization: ${auth === "(none)" ? "(none)" : "[REDACTED]"}`);

  if (req.method === "POST" && url.startsWith("/v1/chat/completions")) {
    let body;
    try {
      body = JSON.parse(await readBody(req));
    } catch {
      res.writeHead(400, { "content-type": "application/json" });
      res.end(JSON.stringify({ error: "invalid json" }));
      return;
    }
    const model = body.model ?? "mock-gpt";

    if (body.stream === true) {
      // ── SSE 流式响应：逐块 + 延迟，证明代理无缓冲 ──
      const pieces = ["TokenHusk", " mock", " SSE", " passthrough", " works"];
      res.writeHead(200, {
        "content-type": "text/event-stream",
        "cache-control": "no-cache",
        connection: "keep-alive",
        // 透传一个自定义头，验证 Rule 5（Header 透传完整性）
        "x-mock-upstream": "ok",
      });
      for (const p of pieces) {
        res.write(`data: ${JSON.stringify(sseChunk(p, model))}\n\n`);
        await sleep(50); // 50ms/块 = 可观测的逐块到达
      }
      res.write(`data: ${JSON.stringify(sseChunk("", model, true))}\n\n`);
      res.write("data: [DONE]\n\n");
      res.end();
      return;
    }

    // ── 非流式响应 ──
    const completion = {
      id: "chatcmpl-mock",
      object: "chat.completion",
      created: 1_700_000_000,
      model,
      choices: [
        {
          index: 0,
          message: { role: "assistant", content: "TokenHusk mock non-stream reply" },
          finish_reason: "stop",
        },
      ],
      usage: { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 },
    };
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify(completion));
    return;
  }

  res.writeHead(404, { "content-type": "application/json" });
  res.end(JSON.stringify({ error: "not found" }));
});

server.listen(PORT, HOST, () => {
  console.log(`[mock] LLM server listening on http://${HOST}:${PORT}`);
  console.log(`[mock] POST /v1/chat/completions  (stream:true -> SSE, else -> JSON)`);
});
