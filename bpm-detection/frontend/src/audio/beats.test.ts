import { describe, expect, test } from "bun:test";
import { bpmToLag, isFlashing, lagToBpm, lastBeatIndex } from "./beats";

const beats = [0.5, 1.0, 1.5, 2.0];

describe("beat indicator timing", () => {
  test("no beat has happened before the first detected beat, so nothing may flash", () => {
    expect(lastBeatIndex(beats, 0.2)).toBe(-1);
    expect(isFlashing(beats, 0.2)).toBe(false);
  });

  test("the indicator flashes right after a beat so the eye sees it on the beat", () => {
    expect(lastBeatIndex(beats, 1.05)).toBe(1);
    expect(isFlashing(beats, 1.05)).toBe(true);
  });

  test("the indicator goes dark between beats, otherwise it would stay lit at fast tempos", () => {
    expect(isFlashing(beats, 1.3)).toBe(false);
  });

  test("a beat exactly at the playhead counts as the current beat", () => {
    expect(lastBeatIndex(beats, 2.0)).toBe(3);
  });
});

describe("lag and bpm conversion", () => {
  test("half a second of frames is 120 BPM and converts back to the same lag", () => {
    const frameRate = 172.265625;
    const lag = bpmToLag(120, frameRate);
    expect(lag).toBeCloseTo(86.13, 2);
    expect(lagToBpm(lag, frameRate)).toBeCloseTo(120, 6);
  });
});
