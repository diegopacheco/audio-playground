import { describe, expect, test } from "bun:test";
import { float32ToBytes, randomClip } from "./pcm";
import type { TrackInfo } from "../types";

const track = (id: number, duration: number): TrackInfo => ({
  id,
  name: `t${id}`,
  duration_sec: duration,
  hashes: 100,
  builtin: true,
});

describe("float32ToBytes", () => {
  test("encodes little endian so the Rust backend decodes the same samples", () => {
    const bytes = float32ToBytes(new Float32Array([0.5, -0.25]));
    const view = new DataView(bytes);
    expect(bytes.byteLength).toBe(8);
    expect(view.getFloat32(0, true)).toBe(0.5);
    expect(view.getFloat32(4, true)).toBe(-0.25);
  });
});

describe("randomClip", () => {
  test("clip always fits inside the chosen track so the backend never rejects it", () => {
    const tracks = [track(0, 30), track(1, 20)];
    for (const r of [0, 0.25, 0.5, 0.999]) {
      const clip = randomClip(tracks, 5, 0.5, () => r);
      expect(clip).not.toBeNull();
      const chosen = tracks.find((t) => t.id === clip?.track);
      expect(clip!.start + clip!.len).toBeLessThanOrEqual(chosen!.duration_sec);
      expect(clip!.start).toBeGreaterThanOrEqual(0);
    }
  });

  test("tracks shorter than the clip are skipped", () => {
    expect(randomClip([track(0, 3)], 5, 0, () => 0.5)).toBeNull();
    expect(randomClip([track(0, 3), track(7, 30)], 5, 0, () => 0.1)?.track).toBe(7);
  });
});
