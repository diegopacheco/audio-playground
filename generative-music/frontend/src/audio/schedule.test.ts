import { describe, expect, test } from "bun:test";
import type { Song } from "../types";
import { beatsToSeconds, buildSchedule, midiToFrequency, songSeconds } from "./schedule";

const song: Song = {
  bpm: 120,
  bars: 1,
  notes: [
    { pitch: 69, start: 0, duration: 1, velocity: 127 },
    { pitch: 81, start: 2, duration: 2, velocity: 64 },
  ],
  chords: [{ symbol: "I", root: 48, pitches: [48, 52, 55], start: 0, duration: 4 }],
};

describe("schedule", () => {
  test("A4 is 440 Hz and an octave doubles, so playback matches the MIDI export pitch", () => {
    expect(midiToFrequency(69)).toBeCloseTo(440);
    expect(midiToFrequency(81)).toBeCloseTo(880);
  });

  test("at 120 BPM a beat lasts half a second so the browser and WAV render share timing", () => {
    expect(beatsToSeconds(1, 120)).toBe(0.5);
    expect(songSeconds(song)).toBe(2);
  });

  test("every melody note and every chord voice becomes one scheduled tone in time order", () => {
    const tones = buildSchedule(song);
    expect(tones.length).toBe(5);
    expect(tones.filter((t) => t.voice === "pad").length).toBe(3);
    const times = tones.map((t) => t.time);
    expect([...times].sort((a, b) => a - b)).toEqual(times);
    const second = tones.find((t) => t.voice === "lead" && t.time === 1);
    expect(second?.frequency).toBeCloseTo(880);
  });

  test("velocity scales loudness so accented downbeats are louder", () => {
    const leads = buildSchedule(song).filter((t) => t.voice === "lead");
    expect(leads[0].gain).toBeGreaterThan(leads[1].gain);
  });
});
