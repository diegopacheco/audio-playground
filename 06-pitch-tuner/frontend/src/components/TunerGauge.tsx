import { MAX_GAUGE_CENTS, MAX_NEEDLE_DEGREES, isInTune, needleAngle } from "../audio/notes";

interface Props {
  cents: number | null;
}

const CENTER_X = 160;
const CENTER_Y = 170;
const RADIUS = 130;
const TICKS = [-50, -40, -30, -20, -10, 0, 10, 20, 30, 40, 50];

function point(degrees: number, radius: number): { x: number; y: number } {
  const radians = (degrees * Math.PI) / 180;
  return { x: CENTER_X + radius * Math.sin(radians), y: CENTER_Y - radius * Math.cos(radians) };
}

export function TunerGauge({ cents }: Props) {
  const angle = needleAngle(cents);
  const tuned = isInTune(cents);
  const arcStart = point(-MAX_NEEDLE_DEGREES, RADIUS);
  const arcEnd = point(MAX_NEEDLE_DEGREES, RADIUS);
  const tip = point(angle, RADIUS - 12);
  return (
    <svg className="gauge" viewBox="0 0 320 200" role="img" aria-label="cents gauge">
      <path
        d={`M ${arcStart.x} ${arcStart.y} A ${RADIUS} ${RADIUS} 0 0 1 ${arcEnd.x} ${arcEnd.y}`}
        className="gauge-arc"
      />
      {TICKS.map((tick) => {
        const degrees = (tick / MAX_GAUGE_CENTS) * MAX_NEEDLE_DEGREES;
        const outer = point(degrees, RADIUS);
        const inner = point(degrees, tick === 0 ? RADIUS - 22 : RADIUS - 12);
        const label = point(degrees, RADIUS + 14);
        return (
          <g key={tick}>
            <line x1={inner.x} y1={inner.y} x2={outer.x} y2={outer.y} className="gauge-tick" />
            <text x={label.x} y={label.y} className="gauge-label">
              {tick > 0 ? `+${tick}` : tick}
            </text>
          </g>
        );
      })}
      <line
        x1={CENTER_X}
        y1={CENTER_Y}
        x2={tip.x}
        y2={tip.y}
        className={tuned ? "needle needle-tuned" : "needle"}
      />
      <circle cx={CENTER_X} cy={CENTER_Y} r={7} className="needle-hub" />
    </svg>
  );
}
