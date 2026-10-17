import { fillLevel } from "../latency";
import type { EngineStatus } from "../types";

interface Props {
  status: EngineStatus | null;
}

const LABELS = { starving: "near underrun", healthy: "healthy", flooding: "near overrun" };

export function RingGauge({ status }: Props) {
  const ratio = status?.ring_fill_ratio ?? 0;
  const fill = status?.counters?.ring_fill ?? 0;
  const capacity = status?.ring_capacity ?? 0;
  const frames = status?.stream?.buffer_frames ?? 0;
  const level = fillLevel(ratio, frames, capacity);
  const pct = Math.min(100, ratio * 100);

  return (
    <div className="card">
      <div className="card-head">
        <h2>Ring buffer fill</h2>
        <span className={`badge ${status?.running ? level : "idle"}`}>
          {status?.running ? LABELS[level] : "idle"}
        </span>
      </div>
      <div className="gauge" role="meter" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100}>
        <div className={`gauge-fill ${level}`} style={{ width: `${pct}%` }} />
      </div>
      <p className="muted">
        {fill} / {capacity} samples ({pct.toFixed(1)}%)
      </p>
    </div>
  );
}
