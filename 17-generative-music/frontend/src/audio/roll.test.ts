import { describe, expect, test } from "bun:test";
import type { Song } from "../types";
import { isBlackKey, pitchName, pitchRect, rollLayout } from "./roll";

const song: Song = {
  bpm: 100,
  bars: 2,
  notes: [
    { pitch: 60, start: 0, duration: 2, velocity: 100 },
    { pitch: 72, start: 4, duration: 4, velocity: 100 },
  ],
  chords: [{ symbol: "I", root: 48, pitches: [48, 52, 55], start: 0, duration: 4 }],
};

describe("piano roll layout", () => {
  test("pitch range covers melody and chords with padding so nothing is clipped", () => {
    const layout = rollLayout(song, 800, 270);
    expect(layout.low).toBe(46);
    expect(layout.high).toBe(74);
    expect(layout.beats).toBe(8);
  });

  test("higher pitches draw higher and later notes draw further right", () => {
    const layout = rollLayout(song, 800, 270);
    const low = pitchRect(layout, 60, 0, 2);
    const high = pitchRect(layout, 72, 4, 4);
    expect(high.y).toBeLessThan(low.y);
    expect(high.x).toBe(400);
    expect(high.width).toBe(400);
    expect(low.width).toBe(200);
  });

  test("the last beat ends exactly at the canvas edge", () => {
    const layout = rollLayout(song, 800, 270);
    const r = pitchRect(layout, 72, 4, 4);
    expect(r.x + r.width).toBe(800);
    const top = pitchRect(layout, layout.high, 0, 1);
    expect(top.y).toBe(0);
  });

  test("note names and black keys follow the chromatic scale", () => {
    expect(pitchName(60)).toBe("C4");
    expect(pitchName(69)).toBe("A4");
    expect(isBlackKey(61)).toBe(true);
    expect(isBlackKey(64)).toBe(false);
  });
});
