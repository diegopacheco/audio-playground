import type { Column, DiffStats } from "../types";

export function diffSamples(original: Int16Array, stego: Int16Array): Int16Array {
  const n = Math.min(original.length, stego.length);
  const out = new Int16Array(n);
  for (let i = 0; i < n; i++) out[i] = stego[i] - original[i];
  return out;
}

export function diffStats(original: Int16Array, stego: Int16Array): DiffStats {
  const n = Math.min(original.length, stego.length);
  let maxDiff = 0;
  let changed = 0;
  for (let i = 0; i < n; i++) {
    const d = stego[i] - original[i];
    if (d !== 0) changed++;
    maxDiff = Math.max(maxDiff, Math.abs(d));
  }
  return { maxDiff, changed };
}

export function columns(samples: Int16Array, width: number): Column[] {
  const out: Column[] = [];
  if (samples.length === 0 || width <= 0) return out;
  const per = samples.length / width;
  for (let x = 0; x < width; x++) {
    const from = Math.floor(x * per);
    const to = Math.max(from + 1, Math.floor((x + 1) * per));
    let min = Infinity;
    let max = -Infinity;
    for (let i = from; i < Math.min(to, samples.length); i++) {
      min = Math.min(min, samples[i]);
      max = Math.max(max, samples[i]);
    }
    out.push({ min: min === Infinity ? 0 : min, max: max === -Infinity ? 0 : max });
  }
  return out;
}

export function utf8Length(text: string): number {
  return new TextEncoder().encode(text).length;
}
