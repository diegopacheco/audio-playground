import { describe, expect, test } from "bun:test";
import { MAX_OCTAVE, MIN_OCTAVE, keyToMidi, keyToSemitone, shiftOctave } from "./keyboard";

describe("QWERTY mapping covers two chromatic octaves", () => {
  test("Z is the root and Q is exactly one octave above it", () => {
    expect(keyToSemitone("z")).toBe(0);
    expect(keyToSemitone("q")).toBe(12);
    expect(keyToSemitone("u")).toBe(23);
  });

  test("black keys sit on the row above the white keys", () => {
    expect(keyToSemitone("s")).toBe(1);
    expect(keyToSemitone("2")).toBe(13);
  });

  test("octave 3 puts Q on middle C and the mapping ignores caps lock", () => {
    expect(keyToMidi("q", 3)).toBe(60);
    expect(keyToMidi("Q", 3)).toBe(60);
    expect(keyToMidi("z", 4)).toBe(60);
  });

  test("unmapped keys produce no note", () => {
    expect(keyToMidi("a", 3)).toBeNull();
  });

  test("octave shift is clamped so notes stay in the playable range", () => {
    expect(shiftOctave(MAX_OCTAVE, 1)).toBe(MAX_OCTAVE);
    expect(shiftOctave(MIN_OCTAVE, -1)).toBe(MIN_OCTAVE);
    expect(shiftOctave(3, 1)).toBe(4);
  });
});
