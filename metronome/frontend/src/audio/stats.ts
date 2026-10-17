import type { JitterStats } from "../types";

export function jitterStats(errors: number[]): JitterStats {
  const count = errors.length;
  if (count === 0) return { count: 0, meanMs: 0, maxAbsMs: 0, stdDevMs: 0 };
  const meanMs = errors.reduce((a, b) => a + b, 0) / count;
  const maxAbsMs = errors.reduce((a, b) => Math.max(a, Math.abs(b)), 0);
  const variance = errors.reduce((a, b) => a + (b - meanMs) ** 2, 0) / count;
  return { count, meanMs, maxAbsMs, stdDevMs: Math.sqrt(variance) };
}
