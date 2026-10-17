import { bucketMax, project, toPath, type Scale } from "../audio/chart";

export interface Marker {
  x: number;
  color: string;
  label?: string;
  dashed?: boolean;
}

interface Props {
  title: string;
  subtitle: string;
  values: number[];
  xMax: number;
  yMin: number;
  yMax: number;
  markers: Marker[];
  color: string;
  xLabels: [string, string];
}

const WIDTH = 900;
const HEIGHT = 180;

export function LineChart({ title, subtitle, values, xMax, yMin, yMax, markers, color, xLabels }: Props) {
  const scale: Scale = { width: WIDTH, height: HEIGHT, xMax, yMin, yMax };
  const path = toPath(bucketMax(values, WIDTH), scale);
  const zeroY = project({ x: 0, y: 0 }, scale).y;
  return (
    <section className="card">
      <header className="card-head">
        <h2>{title}</h2>
        <span className="muted">{subtitle}</span>
      </header>
      <svg viewBox={`0 0 ${WIDTH} ${HEIGHT + 20}`} className="chart" preserveAspectRatio="none">
        {yMin < 0 && <line x1={0} x2={WIDTH} y1={zeroY} y2={zeroY} className="axis" />}
        {markers.map((m, i) => {
          const x = project({ x: m.x, y: 0 }, scale).x;
          return (
            <g key={i}>
              <line
                x1={x}
                x2={x}
                y1={0}
                y2={HEIGHT}
                stroke={m.color}
                strokeWidth={m.label ? 2 : 1}
                strokeDasharray={m.dashed ? "4 4" : undefined}
              />
              {m.label && (
                <text x={x + 4} y={14} fill={m.color} className="marker-label">
                  {m.label}
                </text>
              )}
            </g>
          );
        })}
        <path d={path} fill="none" stroke={color} strokeWidth={1.5} />
        <text x={0} y={HEIGHT + 16} className="tick">
          {xLabels[0]}
        </text>
        <text x={WIDTH} y={HEIGHT + 16} className="tick" textAnchor="end">
          {xLabels[1]}
        </text>
      </svg>
    </section>
  );
}
