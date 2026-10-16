import type { ScheduledTone, Song } from "../types";

const LEAD_GAIN = 0.25;
const PAD_GAIN = 0.05;

export function midiToFrequency(pitch: number): number {
  return 440 * Math.pow(2, (pitch - 69) / 12);
}

export function beatsToSeconds(beats: number, bpm: number): number {
  return (beats * 60) / bpm;
}

export function songSeconds(song: Song): number {
  return beatsToSeconds(song.bars * 4, song.bpm);
}

export function buildSchedule(song: Song): ScheduledTone[] {
  const pads: ScheduledTone[] = song.chords.flatMap((chord) =>
    chord.pitches.map((pitch) => ({
      time: beatsToSeconds(chord.start, song.bpm),
      frequency: midiToFrequency(pitch),
      duration: beatsToSeconds(chord.duration, song.bpm),
      gain: PAD_GAIN,
      voice: "pad" as const,
    })),
  );
  const leads: ScheduledTone[] = song.notes.map((note) => ({
    time: beatsToSeconds(note.start, song.bpm),
    frequency: midiToFrequency(note.pitch),
    duration: beatsToSeconds(note.duration, song.bpm) * 0.95,
    gain: (LEAD_GAIN * note.velocity) / 127,
    voice: "lead",
  }));
  return [...pads, ...leads].sort((a, b) => a.time - b.time);
}
