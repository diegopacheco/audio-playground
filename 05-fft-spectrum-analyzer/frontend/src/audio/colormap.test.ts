import { expect, test } from "bun:test";
import { MAX_DB, MIN_DB, dbToUnit, heatColor } from "./colormap";

test("dB values outside the display range clamp so the floor and clipping stay visible", () => {
  expect(dbToUnit(-120)).toBe(0);
  expect(dbToUnit(MIN_DB)).toBe(0);
  expect(dbToUnit(MAX_DB)).toBe(1);
  expect(dbToUnit(12)).toBe(1);
  expect(dbToUnit(-50)).toBeCloseTo(0.5);
});

test("quiet bins render near white on the light theme and loud bins are clearly darker or warmer", () => {
  const quiet = heatColor(0);
  const loud = heatColor(0.6);
  const brightness = (c: number[]) => c[0] + c[1] + c[2];
  expect(brightness(quiet)).toBeGreaterThan(720);
  expect(brightness(loud)).toBeLessThan(brightness(quiet) - 200);
});

test("color ramp is continuous so neighbouring levels never jump", () => {
  for (let i = 0; i < 100; i++) {
    const a = heatColor(i / 100);
    const b = heatColor((i + 1) / 100);
    const jump = Math.max(...a.map((v, c) => Math.abs(v - b[c])));
    expect(jump).toBeLessThan(40);
  }
});
