import { HEADER_LEN } from "./wavHeader";

export function pcmSamples(buffer: ArrayBuffer, dataSize: number): Int16Array {
  const view = new DataView(buffer, HEADER_LEN, dataSize);
  const out = new Int16Array(dataSize / 2);
  for (let i = 0; i < out.length; i++) out[i] = view.getInt16(i * 2, true);
  return out;
}

export function visibleSampleCount(sampleRate: number, frequency: number, periods: number, total: number): number {
  return Math.min(total, Math.max(2, Math.round((periods * sampleRate) / frequency)));
}

export function toPoints(samples: Int16Array, count: number, width: number, height: number): Array<[number, number]> {
  const mid = height / 2;
  const step = count > 1 ? width / (count - 1) : 0;
  const points: Array<[number, number]> = [];
  for (let i = 0; i < count; i++) {
    points.push([i * step, mid - (samples[i] / 32768) * mid]);
  }
  return points;
}
