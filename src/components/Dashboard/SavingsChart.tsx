import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell } from "recharts";
import type { SavingsSource } from "../../types/ipc";

const data: SavingsSource[] = [
  { stage: "JSON 压缩", saved_tokens: 45, color: "#1E40AF" },
  { stage: "日志去重", saved_tokens: 25, color: "#3B82F6" },
  { stage: "上下文裁剪", saved_tokens: 18, color: "#D97706" },
  { stage: "输出约束", saved_tokens: 12, color: "#16A34A" },
];

export function SavingsChart() {
  return (
    <div className="card">
      <div className="card-header">
        <h3 className="text-sm font-semibold text-ink">节省来源分析</h3>
      </div>
      <div className="card-body">
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart data={data} layout="vertical">
              <XAxis type="number" tick={{ fontSize: 12 }} unit="%" />
              <YAxis
                type="category"
                dataKey="stage"
                tick={{ fontSize: 12 }}
                width={100}
              />
              <Tooltip formatter={(value: number) => [`${value}%`, "节省占比"]} />
              <Bar dataKey="saved_tokens" radius={[0, 4, 4, 0]}>
                {data.map((entry) => (
                  <Cell key={entry.stage} fill={entry.color} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}