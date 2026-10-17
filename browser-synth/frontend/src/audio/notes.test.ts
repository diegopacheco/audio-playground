import { describe, expect, test } from "bun:test";
import { isBlackKey, midiToFrequency, noteName } from "./notes";

describe("note to frequency mapping keeps the synth in tune", () => {
  test("A4 is concert pitch 440 Hz", () => {
    expect(midiToFrequency(69)).toBeCloseTo(440, 6);
  });

  test("an octave doubles or halves the frequency", () => {
    expect(midiToFrequency(81)).toBeCloseTo(880, 6);
    expect(midiToFrequency(57)).toBeCloseTo(220, 6);
  });

  test("middle C matches equal temperament", () => {
    expect(midiToFrequency(60)).toBeCloseTo(261.6256, 3);
  });

  test("names and black keys follow the piano layout", () => {
    expect(noteName(60)).toBe("C4");
    expect(noteName(69)).toBe("A4");
    expect(isBlackKey(61)).toBe(true);
    expect(isBlackKey(64)).toBe(false);
  });
});
