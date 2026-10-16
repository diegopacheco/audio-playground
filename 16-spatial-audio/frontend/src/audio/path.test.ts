import { describe, expect, test } from "bun:test";
import { pointAt } from "./path";
import type { PathPoint } from "../types";

function point(t: number, azimuth: number, distance = 1): PathPoint {
  return { t, azimuth, elevation: 0, distance, itd_ms: 0, ild_db: 0, ild_probe_hz: 4000 };
}

describe("path interpolation for the playback marker", () => {
  test("interpolates linearly between samples", () => {
    const p = pointAt([point(0, 0, 1), point(1, 90, 3)], 0.5);
    expect(p?.azimuth).toBeCloseTo(45, 6);
    expect(p?.distance).toBeCloseTo(2, 6);
  });

  test("crossing behind the head goes the short way through 180 instead of sweeping through the front", () => {
    const p = pointAt([point(0, 170), point(1, -170)], 0.5);
    expect(Math.abs(p?.azimuth ?? 0)).toBeCloseTo(180, 6);
  });

  test("clamps outside the rendered time range", () => {
    const path = [point(0, 10), point(2, 20)];
    expect(pointAt(path, -1)?.azimuth).toBe(10);
    expect(pointAt(path, 5)?.azimuth).toBe(20);
    expect(pointAt([], 1)).toBeNull();
  });
});
