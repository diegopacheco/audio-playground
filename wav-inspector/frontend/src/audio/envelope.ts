import type { Point } from "../types";

export function envelopeOutline(envelope: number[], width: number, height: number): Point[] {
  if (envelope.length === 0) return [];
  const mid = height / 2;
  const step = envelope.length > 1 ? width / (envelope.length - 1) : 0;
  const clamp = (v: number) => Math.min(1, Math.max(0, v));
  const top = envelope.map((v, i) => ({ x: i * step, y: mid - clamp(v) * mid }));
  const bottom = envelope.map((v, i) => ({ x: i * step, y: mid + clamp(v) * mid })).reverse();
  return [...top, ...bottom];
}
