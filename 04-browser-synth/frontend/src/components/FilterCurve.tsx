import { lowpassCoefficients, magnitudeAt } from "../audio/biquad";

interface Props {
  cutoff: number;
  resonance: number;
}

const WIDTH = 240;
const HEIGHT = 80;
const SAMPLE_RATE = 44100;
const MIN_DB = -36;
const MAX_DB = 24;

export function FilterCurve({ cutoff, resonance }: Props) {
  const coeffs = lowpassCoefficients(cutoff, resonance, SAMPLE_RATE);
  const points = Array.from({ length: WIDTH }, (_, x) => {
    const freq = 20 * Math.pow(1000, x / (WIDTH - 1));
    const db = 20 * Math.log10(Math.max(magnitudeAt(coeffs, freq, SAMPLE_RATE), 1e-6));
    const clamped = Math.min(MAX_DB, Math.max(MIN_DB, db));
    const y = ((MAX_DB - clamped) / (MAX_DB - MIN_DB)) * HEIGHT;
    return `${x},${y.toFixed(1)}`;
  });
  const zeroY = (MAX_DB / (MAX_DB - MIN_DB)) * HEIGHT;
  return (
    <svg className="curve" viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="img" aria-label="filter response">
      <line x1="0" x2={WIDTH} y1={zeroY} y2={zeroY} className="zero" />
      <polyline points={points.join(" ")} />
    </svg>
  );
}
