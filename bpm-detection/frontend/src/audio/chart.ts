import type { Point } from "../types";

export function bucketMax(values: number[], buckets: number): Point[] {
  if (values.length <= buckets) return values.map((y, x) => ({ x, y }));
  const size = values.length / buckets;
  const points: Point[] = [];
  for (let b = 0; b < buckets; b++) {
    const start = Math.floor(b * size);
    const end = Math.max(start + 1, Math.floor((b + 1) * size));
    let best = start;
    for (let i = start; i < end; i++) {
      if (values[i] > values[best]) best = i;
    }
    points.push({ x: best, y: values[best] });
  }
  return points;
}

export interface Scale {
  width: number;
  height: number;
  xMax: number;
  yMin: number;
  yMax: number;
}

export function project(point: Point, scale: Scale): Point {
  const ySpan = scale.yMax - scale.yMin || 1;
  return {
    x: (point.x / (scale.xMax || 1)) * scale.width,
    y: scale.height - ((point.y - scale.yMin) / ySpan) * scale.height,
  };
}

export function toPath(points: Point[], scale: Scale): string {
  return points
    .map((p, i) => {
      const q = project(p, scale);
      return `${i === 0 ? "M" : "L"}${q.x.toFixed(1)} ${q.y.toFixed(1)}`;
    })
    .join(" ");
}
