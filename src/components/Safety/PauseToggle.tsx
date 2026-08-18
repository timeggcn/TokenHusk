interface Props {
  paused: boolean;
  onToggle: (paused: boolean) => void;
}

export function PauseToggle({ paused, onToggle }: Props) {
  return (
    <div className="card">
      <div className="card-header">
        <h3 className="text-sm font-semibold text-ink">代理控制</h3>
      </div>
      <div className="card-body space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-ink">
              {paused ? "代理已暂停" : "代理运行中"}
            </p>
            <p className="text-xs text-ink-muted mt-1">
              {paused
                ? "所有请求直接透传，不经过压缩管线"
                : "请求经过压缩管线，自动节省 Token"}
            </p>
          </div>
          <button
            type="button"
            onClick={() => onToggle(!paused)}
            className={`btn min-w-[100px] ${
              paused ? "btn-primary" : "btn-secondary"
            }`}
          >
            {paused ? "恢复运行" : "暂停"}
          </button>
        </div>
      </div>
    </div>
  );
}