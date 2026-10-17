import type { FillLevel, LatencySummary } from "./types";

export const BUFFER_SIZES = [32, 64, 128, 256, 512, 1024, 2048];

export function summarize(values: number[]): LatencySummary | null {
  if (values.length === 0) return null;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const mean = values.reduce((a, b) => a + b, 0) / values.length;
  const variance = values.reduce((a, v) => a + (v - mean) ** 2, 0) / values.length;
  return { count: values.length, min, max, mean, jitter: Math.sqrt(variance) };
}

export function fillLevel(ratio: number, bufferFrames: number, capacity: number): FillLevel {
  if (capacity <= 0) return "healthy";
  const oneBuffer = bufferFrames / capacity;
  if (ratio < oneBuffer * 0.25) return "starving";
  if (ratio > 0.75) return "flooding";
  return "healthy";
}

export function bufferMs(frames: number, sampleRate: number): number {
  return (frames * 1000) / sampleRate;
}

export interface ChartPoint {
  x: number;
  y: number;
  value: number;
}

export function chartPoints(
  values: number[],
  width: number,
  height: number,
  pad: number,
): { points: ChartPoint[]; top: number } {
  const top = niceCeil(Math.max(1, ...values) * 1.2);
  const span = Math.max(1, values.length - 1);
  const points = values.map((value, i) => ({
    x: pad + (values.length === 1 ? (width - 2 * pad) / 2 : (i / span) * (width - 2 * pad)),
    y: height - pad - (value / top) * (height - 2 * pad),
    value,
  }));
  return { points, top };
}

export function niceCeil(v: number): number {
  const exp = Math.pow(10, Math.floor(Math.log10(v)));
  for (const step of [1, 2, 5, 10]) {
    if (step * exp >= v) return step * exp;
  }
  return 10 * exp;
}

export function appendHistory(history: number[], value: number, limit: number): number[] {
  const next = [...history, value];
  return next.length > limit ? next.slice(next.length - limit) : next;
}
