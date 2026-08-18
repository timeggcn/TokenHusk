import type { DashboardOverview } from "../../types/ipc";

interface Props {
  overview: DashboardOverview;
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toString();
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  return `${h}h ${m}m ${s}s`;
}

export function Overview({ overview }: Props) {
  const cards = [
    {
      label: "今日请求",
      value: overview.today_requests.toString(),
      sub: `累计 ${overview.total_requests_all_time} 次`,
      color: "text-primary",
    },
    {
      label: "节省 Token",
      value: formatTokens(overview.today_saved_tokens),
      sub: `${(overview.today_saved_ratio * 100).toFixed(1)}% 压缩率`,
      color: "text-success",
    },
    {
      label: "节省费用",
      value: `$${overview.today_saved_cost.toFixed(2)}`,
      sub: `预估总费用 $${overview.today_estimated_cost.toFixed(2)}`,
      color: "text-accent",
    },
    {
      label: "代理状态",
      value: overview.proxy_running ? "运行中" : "已停止",
      sub: `已运行 ${formatDuration(overview.proxy_uptime_seconds)}`,
      color: overview.proxy_running ? "text-success" : "text-destructive",
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      {cards.map((card) => (
        <div key={card.label} className="card">
          <div className="card-body">
            <p className="text-xs font-medium tracking-wider text-ink-subtle uppercase">
              {card.label}
            </p>
            <p className={`mt-1 text-2xl font-semibold ${card.color}`}>
              {card.value}
            </p>
            <p className="mt-1 text-xs text-ink-muted">{card.sub}</p>
          </div>
        </div>
      ))}
    </div>
  );
}