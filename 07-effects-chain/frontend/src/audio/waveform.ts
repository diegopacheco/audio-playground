import type { Peak } from "../types";

export function computePeaks(samples: Float32Array, buckets: number): Peak[] {
  if (samples.length === 0 || buckets <= 0) return [];
  const size = samples.length / buckets;
  const peaks: Peak[] = [];
  for (let b = 0; b < buckets; b++) {
    const start = Math.floor(b * size);
    const end = Math.max(start + 1, Math.floor((b + 1) * size));
    let min = 0;
    let max = 0;
    for (let i = start; i < end && i < samples.length; i++) {
      if (samples[i] < min) min = samples[i];
      if (samples[i] > max) max = samples[i];
    }
    peaks.push({ min, max });
  }
  return peaks;
}
