const LOWER_ROW = ["z", "s", "x", "d", "c", "v", "g", "b", "h", "n", "j", "m"];
const UPPER_ROW = ["q", "2", "w", "3", "e", "r", "5", "t", "6", "y", "7", "u"];

export const MIN_OCTAVE = 1;
export const MAX_OCTAVE = 6;
export const OCTAVE_DOWN_KEY = "[";
export const OCTAVE_UP_KEY = "]";

export function keyToSemitone(key: string): number | null {
  const k = key.toLowerCase();
  const lower = LOWER_ROW.indexOf(k);
  if (lower >= 0) return lower;
  const upper = UPPER_ROW.indexOf(k);
  return upper >= 0 ? upper + 12 : null;
}

export function keyToMidi(key: string, octave: number): number | null {
  const semitone = keyToSemitone(key);
  return semitone === null ? null : 12 * (octave + 1) + semitone;
}

export function semitoneToKey(semitone: number): string {
  return semitone < 12 ? LOWER_ROW[semitone] : UPPER_ROW[semitone - 12];
}

export function shiftOctave(octave: number, delta: number): number {
  return Math.min(MAX_OCTAVE, Math.max(MIN_OCTAVE, octave + delta));
}
