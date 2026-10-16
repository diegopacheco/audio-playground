import { useState } from "react";
import type { ChartSeries } from "../types";

interface Props {
  title: string;
  series: ChartSeries[];
  unit: string;
}

const W = 640;
const H = 220;
const PAD = { left: 48, right: 120, top: 16, bottom: 28 };
const LABEL_GAP = 14;

function spreadLabels(ends: { name: string; y: number }[]): Map<string, number> {
  const sorted = [...ends].sort((a, b) => a.y - b.y);
  const placed = new Map<string, number>();
  let prev = -Infinity;
  for (const end of sorted) {
    const at = Math.max(end.y, prev + LABEL_GAP);
    placed.set(end.name, at);
    prev = at;
  }
  return placed;
}

function niceMax(v: number): number {
  if (v <= 1) return 1;
  const p = 10 ** Math.floor(Math.log10(v));
  const n = v / p;
  return (n <= 2 ? 2 : n <= 5 ? 5 : 10) * p;
}

export function JitterChart({ title, series, unit }: Props) {
  const [hover, setHover] = useState<number | null>(null);
  const length = Math.max(0, ...series.map((s) => s.values.length));
  const all = series.flatMap((s) => s.values);
  const top = niceMax(Math.max(0, ...all));
  const minV = Math.min(0, ...all);
  const lo = minV < 0 ? -niceMax(-minV) : 0;
  const plotW = W - PAD.left - PAD.right;
  const plotH = H - PAD.top - PAD.bottom;
  const x = (i: number) => PAD.left + (length <= 1 ? 0 : (i / (length - 1)) * plotW);
  const y = (v: number) => PAD.top + ((top - v) / (top - lo)) * plotH;
  const ticks = lo === 0 ? [0, top / 2, top] : [lo, 0, top];
  const labelY = spreadLabels(
    series.filter((s) => s.values.length > 0).map((s) => ({ name: s.name, y: y(s.values[s.values.length - 1] ?? 0) })),
  );

  const onMove = (e: React.MouseEvent<SVGRectElement>) => {
    if (length === 0) return;
    const rect = e.currentTarget.getBoundingClientRect();
    const ratio = (e.clientX - rect.left) / rect.width;
    setHover(Math.max(0, Math.min(length - 1, Math.round(ratio * (length - 1)))));
  };

  return (
    <figure className="chart">
      <figcaption>
        <span className="chart-title">{title}</span>
        <span className="legend">
          {series.map((s) => (
            <span key={s.name} className="legend-item">
              <span className="swatch" style={{ background: s.color }} />
              {s.name}
            </span>
          ))}
        </span>
      </figcaption>
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={title}>
        {ticks.map((t) => (
          <g key={t}>
            <line x1={PAD.left} x2={W - PAD.right} y1={y(t)} y2={y(t)} className={t === 0 ? "axis" : "grid"} />
            <text x={PAD.left - 6} y={y(t) + 4} textAnchor="end" className="tick">
              {t.toFixed(t % 1 === 0 ? 0 : 1)}
            </text>
          </g>
        ))}
        <text x={PAD.left} y={H - 6} className="tick">beat</text>
        <text x={W - PAD.right} y={H - 6} textAnchor="end" className="tick">{unit}</text>
        {series.map((s) =>
          s.values.length > 0 ? (
            <g key={s.name}>
              <polyline
                fill="none"
                stroke={s.color}
                strokeWidth={2}
                strokeLinejoin="round"
                points={s.values.map((v, i) => `${x(i)},${y(v)}`).join(" ")}
              />
              <text
                x={W - PAD.right + 8}
                y={(labelY.get(s.name) ?? 0) + 4}
                className="direct-label"
              >
                {s.name}
              </text>
            </g>
          ) : null,
        )}
        {hover !== null && (
          <g>
            <line x1={x(hover)} x2={x(hover)} y1={PAD.top} y2={H - PAD.bottom} className="crosshair" />
            {series.map((s) => {
              const v = s.values[hover];
              return v === undefined ? null : (
                <circle key={s.name} cx={x(hover)} cy={y(v)} r={4} fill={s.color} stroke="#fff" strokeWidth={2} />
              );
            })}
          </g>
        )}
        <rect
          x={PAD.left}
          y={PAD.top}
          width={plotW}
          height={plotH}
          fill="transparent"
          onMouseMove={onMove}
          onMouseLeave={() => setHover(null)}
        />
      </svg>
      {hover !== null && (
        <div className="tooltip">
          <strong>beat {hover}</strong>
          {series.map((s) => (
            <span key={s.name}>
              <span className="swatch" style={{ background: s.color }} />
              {s.name}: {(s.values[hover] ?? 0).toFixed(2)} {unit}
            </span>
          ))}
        </div>
      )}
    </figure>
  );
}
