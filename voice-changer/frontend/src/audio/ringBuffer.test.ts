import { describe, expect, test } from "bun:test";
import { JitterBuffer, RingBuffer } from "./ringBuffer";

describe("RingBuffer", () => {
  test("keeps samples in order across the wrap point so playback has no reordering clicks", () => {
    const ring = new RingBuffer(4);
    ring.push(new Float32Array([1, 2, 3]));
    const first = new Float32Array(2);
    ring.pull(first);
    ring.push(new Float32Array([4, 5, 6]));
    const rest = new Float32Array(4);
    expect(ring.pull(rest)).toBe(4);
    expect(Array.from(first)).toEqual([1, 2]);
    expect(Array.from(rest)).toEqual([3, 4, 5, 6]);
  });

  test("overwrites the oldest samples when full so latency cannot grow without bound", () => {
    const ring = new RingBuffer(3);
    ring.push(new Float32Array([1, 2, 3, 4, 5]));
    const out = new Float32Array(3);
    ring.pull(out);
    expect(Array.from(out)).toEqual([3, 4, 5]);
  });

  test("pads with silence on underrun instead of replaying stale audio", () => {
    const ring = new RingBuffer(8);
    ring.push(new Float32Array([0.5]));
    const out = new Float32Array([9, 9, 9]);
    expect(ring.pull(out)).toBe(1);
    expect(Array.from(out)).toEqual([0.5, 0, 0]);
  });
});

describe("JitterBuffer", () => {
  test("stays silent until the prebuffer target is reached, then plays gaplessly", () => {
    const jitter = new JitterBuffer(64, 4, 32);
    const out = new Float32Array(2);
    jitter.push(new Float32Array([1, 2]));
    jitter.render(out);
    expect(Array.from(out)).toEqual([0, 0]);
    jitter.push(new Float32Array([3, 4]));
    jitter.render(out);
    expect(Array.from(out)).toEqual([1, 2]);
    jitter.push(new Float32Array([5, 6]));
    jitter.render(out);
    expect(Array.from(out)).toEqual([3, 4]);
  });

  test("re-primes after an underrun so network jitter becomes one gap, not constant crackle", () => {
    const jitter = new JitterBuffer(64, 4, 32);
    const out = new Float32Array(4);
    jitter.push(new Float32Array([1, 2, 3, 4, 5]));
    jitter.render(out);
    jitter.render(out);
    expect(Array.from(out)).toEqual([5, 0, 0, 0]);
    jitter.push(new Float32Array([6, 7]));
    jitter.render(out);
    expect(Array.from(out)).toEqual([0, 0, 0, 0]);
  });

  test("drops back to the target when too much audio piles up, bounding latency", () => {
    const jitter = new JitterBuffer(64, 4, 10);
    jitter.push(new Float32Array(12).fill(1));
    expect(jitter.buffered).toBe(4);
  });
});
