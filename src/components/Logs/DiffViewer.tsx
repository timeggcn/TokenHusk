import { useRef, useMemo } from "react";
import { FixedSizeList as List } from "react-window";
import type { RequestRecord } from "../../types/ipc";

interface Props {
  record: RequestRecord;
}

function tokenizeBody(body: string): string[] {
  return body.split("\n");
}

function diffLines(original: string[], compressed: string[]) {
  const maxLen = Math.max(original.length, compressed.length);
  const lines: Array<{ type: "same" | "removed" | "added"; text: string; side: "left" | "right" }> = [];
  for (let i = 0; i < maxLen; i++) {
    const left = original[i] ?? "";
    const right = compressed[i] ?? "";
    if (left === right) {
      lines.push({ type: "same", text: left, side: "left" });
      lines.push({ type: "same", text: right, side: "right" });
    } else {
      if (left) lines.push({ type: "removed", text: left, side: "left" });
      if (right) lines.push({ type: "added", text: right, side: "right" });
    }
  }
  return lines;
}

function Row({ index, style, data }: { index: number; style: React.CSSProperties; data: ReturnType<typeof diffLines> }) {
  const line = data[index];
  const bgClass = line.type === "removed" ? "bg-red-50" : line.type === "added" ? "bg-green-50" : "";
  return (
    <div style={style} className={`flex font-mono text-xs leading-6 ${bgClass}`}>
      <span className="w-1/2 px-2 truncate border-r border-line" title={line.text}>
        {line.text || "\u00A0"}
      </span>
      <span className="w-1/2 px-2 truncate" title={line.text}>
        {line.text || "\u00A0"}
      </span>
    </div>
  );
}

export function DiffViewer({ record }: Props) {
  const originalLines = useMemo(() => tokenizeBody(record.original_body), [record.original_body]);
  const compressedLines = useMemo(
    () => tokenizeBody(record.compressed_body ?? record.original_body),
    [record.compressed_body, record.original_body]
  );
  const lines = useMemo(() => diffLines(originalLines, compressedLines), [originalLines, compressedLines]);
  const listRef = useRef<List>(null);

  return (
    <div className="card">
      <div className="card-header flex items-center justify-between">
        <div className="flex items-center gap-4">
          <h3 className="text-sm font-semibold text-ink">Diff 查看器</h3>
          <span className="text-xs text-ink-muted">
            请求 #{record.id}
          </span>
        </div>
        <div className="flex items-center gap-3 text-xs text-ink-muted">
          <span className="flex items-center gap-1">
            <span className="w-3 h-3 rounded bg-red-50 border border-red-200" />
            移除
          </span>
          <span className="flex items-center gap-1">
            <span className="w-3 h-3 rounded bg-green-50 border border-green-200" />
            新增
          </span>
        </div>
      </div>
      <div className="flex border-b border-line bg-slate-50 text-xs text-ink-muted font-medium">
        <div className="w-1/2 px-3 py-2 border-r border-line">原始消息 ({originalLines.length} 行)</div>
        <div className="w-1/2 px-3 py-2">压缩后消息 ({compressedLines.length} 行)</div>
      </div>
      <div className="h-96">
        <List
          ref={listRef}
          height={384}
          itemCount={lines.length}
          itemSize={24}
          itemData={lines}
          width="100%"
        >
          {Row}
        </List>
      </div>
    </div>
  );
}