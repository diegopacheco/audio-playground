import type { ClipRequest, TrackInfo } from "../types";

export function float32ToBytes(samples: Float32Array): ArrayBuffer {
  const out = new ArrayBuffer(samples.length * 4);
  const view = new DataView(out);
  samples.forEach((value, i) => view.setFloat32(i * 4, value, true));
  return out;
}

export function randomClip(
  tracks: TrackInfo[],
  len: number,
  noise: number,
  random: () => number,
): ClipRequest | null {
  const usable = tracks.filter((t) => t.duration_sec > len);
  if (usable.length === 0) {
    return null;
  }
  const track = usable[Math.floor(random() * usable.length) % usable.length];
  const start = random() * (track.duration_sec - len);
  return { track: track.id, start: Math.round(start * 100) / 100, len, noise };
}
