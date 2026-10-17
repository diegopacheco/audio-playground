import { describe, expect, test } from "bun:test";
import { appendHistory, bufferMs, chartPoints, fillLevel, niceCeil, summarize } from "./latency";

describe("summarize", () => {
  test("jitter is zero for a perfectly stable loop and positive when results drift", () => {
    expect(summarize([10, 10, 10])?.jitter).toBe(0);
    const s = summarize([8, 10, 12]);
    expect(s?.mean).toBe(10);
    expect(s?.min).toBe(8);
    expect(s?.max).toBe(12);
    expect(s?.jitter).toBeGreaterThan(1.6);
  });

  test("no measurements yields no summary instead of NaN figures", () => {
    expect(summarize([])).toBeNull();
  });
});

describe("fillLevel", () => {
  test("ring holding less than a quarter buffer is about to underrun", () => {
    expect(fillLevel(10 / 2048, 256, 2048)).toBe("starving");
  });
  test("ring near capacity is about to overrun", () => {
    expect(fillLevel(0.9, 256, 2048)).toBe("flooding");
  });
  test("one buffer of slack is healthy", () => {
    expect(fillLevel(256 / 2048, 256, 2048)).toBe("healthy");
  });
});

describe("chart helpers", () => {
  test("480 frames at 48k is 10 ms of buffering", () => {
    expect(bufferMs(480, 48000)).toBe(10);
  });

  test("larger latency is drawn higher on screen and axis top covers the max", () => {
    const { points, top } = chartPoints([5, 15], 200, 100, 10);
    expect(top).toBe(20);
    expect(points[1].y).toBeLessThan(points[0].y);
    expect(points[0].x).toBe(10);
    expect(points[1].x).toBe(190);
  });

  test("niceCeil rounds up to 1-2-5 steps", () => {
    expect(niceCeil(7)).toBe(10);
    expect(niceCeil(13)).toBe(20);
    expect(niceCeil(41)).toBe(50);
  });

  test("history keeps only the most recent results", () => {
    expect(appendHistory([1, 2, 3], 4, 3)).toEqual([2, 3, 4]);
  });
});
