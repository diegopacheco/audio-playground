import { describe, expect, test } from "bun:test";
import { encodeBase64 } from "./base64";
import { computePeaks } from "./waveform";

describe("waveform peaks", () => {
  test("each bucket keeps its loudest positive and negative sample so transients stay visible", () => {
    const samples = new Float32Array([0, 0.9, -0.2, 0, 0, -0.7, 0.1, 0]);
    expect(computePeaks(samples, 2)).toEqual([
      { min: -0.20000000298023224, max: 0.8999999761581421 },
      { min: -0.699999988079071, max: 0.10000000149011612 },
    ]);
  });

  test("more buckets than samples still yields one peak per bucket for a full-width canvas", () => {
    expect(computePeaks(new Float32Array([0.5, -0.5]), 4)).toHaveLength(4);
  });

  test("silence draws a flat line", () => {
    expect(computePeaks(new Float32Array(100), 5).every((p) => p.min === 0 && p.max === 0)).toBe(true);
  });
});

describe("base64 upload encoding", () => {
  test("matches the standard alphabet the Rust decoder expects", () => {
    expect(encodeBase64(new Uint8Array([77, 97, 110]))).toBe("TWFu");
    expect(encodeBase64(new Uint8Array([77, 97]))).toBe("TWE=");
  });

  test("large buffers are encoded in chunks without overflowing the call stack", () => {
    const bytes = new Uint8Array(200_000).fill(255);
    expect(encodeBase64(bytes).length).toBe(Math.ceil(200_000 / 3) * 4);
  });
});
