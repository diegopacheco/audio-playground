import { describe, expect, test } from "bun:test";
import { INSTRUMENTS, STEPS, emptyPattern, normalize, toggleStep } from "./pattern";

describe("pattern", () => {
  test("empty pattern has 16 silent steps per instrument to match the backend contract", () => {
    const p = emptyPattern();
    for (const name of INSTRUMENTS) {
      expect(p.tracks[name]).toHaveLength(STEPS);
      expect(p.tracks[name].every((on) => !on)).toBe(true);
    }
  });

  test("toggling flips one step and leaves the original untouched so React sees a new state", () => {
    const before = emptyPattern();
    const after = toggleStep(before, "snare", 4);
    expect(after.tracks.snare[4]).toBe(true);
    expect(before.tracks.snare[4]).toBe(false);
    expect(after.tracks.kick).toBe(before.tracks.kick);
    expect(toggleStep(after, "snare", 4).tracks.snare[4]).toBe(false);
  });

  test("normalize fills missing tracks and drops rows with the wrong length", () => {
    const p = normalize({ tracks: { kick: Array(STEPS).fill(true), snare: [true] } });
    expect(p.tracks.kick.every(Boolean)).toBe(true);
    expect(p.tracks.snare).toHaveLength(STEPS);
    expect(p.tracks.snare.some(Boolean)).toBe(false);
    expect(p.tracks.clap).toHaveLength(STEPS);
  });
});
