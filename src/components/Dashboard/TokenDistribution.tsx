import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";
import type { TokenDistribution } from "../../types/ipc";

const data: TokenDistribution[] = [
  { name: "工具输出", value: 58, color: "#1E40AF" },
  { name: "代码上下文", value: 22, color: "#3B82F6" },
  { name: "对话历史", value: 12, color: "#D97706" },
  { name: "用户指令", value: 8, color: "#16A34A" },
];

export function TokenDistributionChart() {
  return (
    <div className="card">
      <div className="card-header">
        <h3 className="text-sm font-semibold text-ink">Token 消耗分布</h3>
      </div>
      <div className="card-body">
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <PieChart>
              <Pie
                data={data}
                cx="50%"
                cy="50%"
                innerRadius={60}
                outerRadius={90}
                paddingAngle={2}
                dataKey="value"
              >
                {data.map((entry) => (
                  <Cell key={entry.name} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip
                formatter={(value: number) => [`${value}%`, "占比"]}
              />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </div>
        <p className="mt-2 text-xs text-ink-muted text-center">
          💡 60% 的 Token 是工具输出结构噪声
        </p>
      </div>
    </div>
  );
}