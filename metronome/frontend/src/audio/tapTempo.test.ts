import { describe, expect, test } from "bun:test";
import { TAP_KEEP, addTap, bpmFromTaps } from "./tapTempo";

describe("tap tempo", () => {
  test("taps half a second apart mean 120 bpm", () => {
    expect(bpmFromTaps([0, 500, 1000, 1500])).toBe(120);
  });

  test("averages uneven taps instead of trusting only the last pair", () => {
    expect(bpmFromTaps([0, 480, 1000, 1500])).toBe(120);
  });

  test("a long pause starts a fresh tempo so old taps do not pollute it", () => {
    expect(addTap([0, 500, 1000], 5000)).toEqual([5000]);
  });

  test("keeps only the most recent taps so tempo changes are followed", () => {
    let taps: number[] = [];
    for (let i = 0; i < 20; i++) taps = addTap(taps, i * 500);
    expect(taps.length).toBe(TAP_KEEP);
  });

  test("one tap or out of range tempos give no bpm", () => {
    expect(bpmFromTaps([100])).toBeNull();
    expect(bpmFromTaps([0, 100])).toBeNull();
  });
});
