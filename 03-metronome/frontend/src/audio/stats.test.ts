import { describe, expect, test } from "bun:test";
import { gridErrorMs } from "./intervalMetronome";
import { jitterStats } from "./stats";

describe("jitter measurement", () => {
  test("a perfect clock has zero jitter", () => {
    expect(jitterStats([0, 0, 0])).toEqual({ count: 3, meanMs: 0, maxAbsMs: 0, stdDevMs: 0 });
  });

  test("max uses absolute error so early beats count as much as late ones", () => {
    expect(jitterStats([1, -5, 2]).maxAbsMs).toBe(5);
  });

  test("std dev separates steady offset from jitter", () => {
    const steady = jitterStats([4, 4, 4, 4]);
    const jittery = jitterStats([0, 8, 0, 8]);
    expect(steady.meanMs).toBe(jittery.meanMs);
    expect(steady.stdDevMs).toBe(0);
    expect(jittery.stdDevMs).toBe(4);
  });

  test("interval error is measured against the ideal beat grid", () => {
    expect(gridErrorMs(1003, 2, 500)).toBe(3);
    expect(gridErrorMs(1498, 3, 500)).toBe(-2);
  });
});
