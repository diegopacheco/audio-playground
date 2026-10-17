import { describe, expect, test } from "bun:test";
import { isInTune, needleAngle, noteFrequency } from "./notes";

describe("noteFrequency", () => {
  test("A4 equals the reference so the tone generator follows the A4 input", () => {
    expect(noteFrequency({ name: "A", octave: 4 }, 440)).toBeCloseTo(440, 9);
    expect(noteFrequency({ name: "A", octave: 4 }, 432)).toBeCloseTo(432, 9);
  });

  test("low E string is 82.41 Hz so guitar tuning presets are correct", () => {
    expect(noteFrequency({ name: "E", octave: 2 }, 440)).toBeCloseTo(82.41, 2);
  });

  test("octaves start at C so C4 is below A4, not above", () => {
    expect(noteFrequency({ name: "C", octave: 4 }, 440)).toBeCloseTo(261.63, 2);
  });

  test("unknown note names fail loudly instead of producing NaN tones", () => {
    expect(() => noteFrequency({ name: "H", octave: 4 }, 440)).toThrow();
  });
});

describe("needleAngle", () => {
  test("no reading parks the needle at center", () => {
    expect(needleAngle(null)).toBe(0);
  });

  test("needle direction follows the sign of cents", () => {
    expect(needleAngle(25)).toBe(30);
    expect(needleAngle(-25)).toBe(-30);
  });

  test("readings past the gauge edge are clamped so the needle never leaves the dial", () => {
    expect(needleAngle(200)).toBe(60);
    expect(needleAngle(-200)).toBe(-60);
  });
});

describe("isInTune", () => {
  test("within five cents counts as in tune", () => {
    expect(isInTune(4.9)).toBe(true);
    expect(isInTune(-5)).toBe(true);
    expect(isInTune(6)).toBe(false);
    expect(isInTune(null)).toBe(false);
  });
});
