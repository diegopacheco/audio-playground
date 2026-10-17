import { expect, test } from "bun:test";
import { Framer } from "./framer";

test("128-sample worklet blocks are regrouped into exact FFT-sized frames without losing samples", () => {
  const framer = new Framer(512);
  const frames: Float32Array[] = [];
  let next = 0;
  for (let b = 0; b < 9; b++) {
    const block = new Float32Array(128).map(() => next++);
    frames.push(...framer.push(block));
  }
  expect(frames.length).toBe(2);
  expect(frames.every((f) => f.length === 512)).toBe(true);
  expect(frames[0][0]).toBe(0);
  expect(frames[1][0]).toBe(512);
  expect(frames[1][511]).toBe(1023);
});

test("a chunk larger than the frame yields several frames and keeps the remainder for later", () => {
  const framer = new Framer(4);
  const out = framer.push(new Float32Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
  expect(out.map((f) => Array.from(f))).toEqual([
    [1, 2, 3, 4],
    [5, 6, 7, 8],
  ]);
  const rest = framer.push(new Float32Array([11, 12]));
  expect(rest.map((f) => Array.from(f))).toEqual([[9, 10, 11, 12]]);
});

test("emitted frames are independent so sending one over the socket is never overwritten", () => {
  const framer = new Framer(2);
  const [a] = framer.push(new Float32Array([1, 2]));
  framer.push(new Float32Array([3, 4]));
  expect(Array.from(a)).toEqual([1, 2]);
});
