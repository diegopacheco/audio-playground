import { useState } from "react";
import { chartPoints, summarize } from "../latency";

interface Props {
  values: number[];
}

const W = 1000;
const H = 240;
const PAD = 32;

export function LatencyChart({ values }: Props) {
  const [hover, setHover] = useState<number | null>(null);
  const summary = summarize(values);
  const { points, top } = chartPoints(values, W, H, PAD);
  const path = points.map((p, i) => `${i === 0 ? "M" : "L"}${p.x},${p.y}`).join(" ");
  const ticks = [0, top / 2, top];
  const active = hover === null ? null : points[hover];

  return (
    <div className="card">
      <div className="card-head">
        <h2>Round-trip latency history</h2>
        {summary && (
          <span className="muted">
            last {values[values.length - 1].toFixed(2)} ms, mean {summary.mean.toFixed(2)} ms, jitter{" "}
            {summary.jitter.toFixed(2)} ms, n={summary.count}
          </span>
        )}
      </div>
      {values.length === 0 ? (
        <p className="muted empty">Start the engine and press Measure latency to record results.</p>
      ) : (
        <svg viewBox={`0 0 ${W} ${H}`} className="chart" onMouseLeave={() => setHover(null)}>
          {ticks.map((t) => {
            const y = H - PAD - (t / top) * (H - 2 * PAD);
            return (
              <g key={t}>
                <line x1={PAD} x2={W - PAD} y1={y} y2={y} className="grid" />
                <text x={PAD - 6} y={y + 4} className="axis" textAnchor="end">
                  {t}
                </text>
              </g>
            );
          })}
          <text x={W - PAD} y={H - 8} className="axis" textAnchor="end">
            measurement
          </text>
          <text x={PAD} y={14} className="axis">
            ms
          </text>
          <path d={path} className="line" />
          {points.map((p, i) => (
            <g key={i} onMouseEnter={() => setHover(i)}>
              <circle cx={p.x} cy={p.y} r={14} className="hit" />
              <circle cx={p.x} cy={p.y} r={4} className="dot" />
            </g>
          ))}
          {active && (
            <g>
              <line x1={active.x} x2={active.x} y1={PAD} y2={H - PAD} className="crosshair" />
              <rect x={Math.min(active.x + 8, W - 128)} y={PAD} width={120} height={24} rx={4} className="tip" />
              <text x={Math.min(active.x + 16, W - 120)} y={PAD + 16} className="tip-text">
                #{(hover ?? 0) + 1}: {active.value.toFixed(2)} ms
              </text>
            </g>
          )}
        </svg>
      )}
    </div>
  );
}
