import { describe, expect, test } from "bun:test";
import { stepDuration, stepsInWindow, swingOffset } from "./timing";

describe("timing", () => {
  test("a step is a sixteenth note so 120 bpm gives 125 ms steps", () => {
    expect(stepDuration(120)).toBeCloseTo(0.125, 9);
    expect(stepDuration(60)).toBeCloseTo(0.25, 9);
  });

  test("swing delays only odd steps by a fraction of the step", () => {
    expect(swingOffset(0, 120, 0.4)).toBe(0);
    expect(swingOffset(2, 120, 0.4)).toBe(0);
    expect(swingOffset(1, 120, 0.4)).toBeCloseTo(0.05, 9);
    expect(swingOffset(3, 120, 0)).toBe(0);
  });

  test("lookahead window schedules every step whose grid time falls before the horizon", () => {
    const batch = stepsInWindow({ nextStep: 0, nextBaseTime: 1 }, 1.3, 120, 0.2);
    expect(batch.steps.map((s) => s.step)).toEqual([0, 1, 2]);
    expect(batch.steps[0].time).toBeCloseTo(1, 9);
    expect(batch.steps[1].time).toBeCloseTo(1.125 + 0.025, 9);
    expect(batch.steps[2].time).toBeCloseTo(1.25, 9);
    expect(batch.cursor.nextStep).toBe(3);
    expect(batch.cursor.nextBaseTime).toBeCloseTo(1.375, 9);
  });

  test("steps wrap after 16 so the pattern loops", () => {
    const batch = stepsInWindow({ nextStep: 15, nextBaseTime: 0 }, 0.2, 120, 0);
    expect(batch.steps.map((s) => s.step)).toEqual([15, 0]);
  });

  test("swing never drifts the grid because the cursor advances on straight time", () => {
    let cursor = { nextStep: 0, nextBaseTime: 0 };
    for (let i = 0; i < 10; i++) cursor = stepsInWindow(cursor, cursor.nextBaseTime + 0.5, 120, 0.5).cursor;
    expect(cursor.nextBaseTime).toBeCloseTo(5, 6);
    expect(cursor.nextStep).toBe(40 % 16);
  });
});
