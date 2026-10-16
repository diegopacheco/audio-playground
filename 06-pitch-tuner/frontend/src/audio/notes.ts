import type { NoteChoice } from "../types";

export const NOTE_NAMES = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

export const MAX_GAUGE_CENTS = 50;
export const MAX_NEEDLE_DEGREES = 60;
export const IN_TUNE_CENTS = 5;

export function noteFrequency(choice: NoteChoice, a4: number): number {
  const index = NOTE_NAMES.indexOf(choice.name);
  if (index < 0) {
    throw new Error(`unknown note ${choice.name}`);
  }
  const midi = (choice.octave + 1) * 12 + index;
  return a4 * Math.pow(2, (midi - 69) / 12);
}

export function needleAngle(cents: number | null): number {
  if (cents === null) {
    return 0;
  }
  const clamped = Math.max(-MAX_GAUGE_CENTS, Math.min(MAX_GAUGE_CENTS, cents));
  return (clamped / MAX_GAUGE_CENTS) * MAX_NEEDLE_DEGREES;
}

export function isInTune(cents: number | null): boolean {
  return cents !== null && Math.abs(cents) <= IN_TUNE_CENTS;
}
