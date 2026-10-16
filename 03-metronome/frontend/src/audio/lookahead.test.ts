import { describe, expect, test } from "bun:test";
import { dueBeats, lateMs } from "./lookahead";

describe("lookahead scheduling", () => {
  test("schedules only the beats that fall inside the lookahead window", () => {
    const beats = dueBeats(0, 1.0, 0.5, 1.0, 0.6);
    expect(beats.map((b) => b.index)).toEqual([0, 1]);
  });

  test("beat times come from the absolute grid so late timer wakeups never shift them", () => {
    const onTime = dueBeats(4, 2.0, 0.25, 2.95, 0.1);
    const lateWake = dueBeats(4, 2.0, 0.25, 3.04, 0.1);
    expect(onTime).toEqual([{ index: 4, time: 3.0 }]);
    expect(lateWake[0]).toEqual({ index: 4, time: 3.0 });
  });

  test("resuming from nextIndex never schedules a beat twice", () => {
    const first = dueBeats(0, 0, 0.5, 0, 0.6);
    const next = first[first.length - 1]!.index + 1;
    const second = dueBeats(next, 0, 0.5, 0.05, 0.6);
    const all = [...first, ...second].map((b) => b.index);
    expect(new Set(all).size).toBe(all.length);
  });

  test("a beat scheduled before its time is not late, one scheduled after is", () => {
    expect(lateMs(3.0, 2.9)).toBe(0);
    expect(lateMs(3.0, 3.02)).toBeCloseTo(20, 6);
  });
});
