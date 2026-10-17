import type { InstrumentName, Pattern, Tracks } from "./types";

export const STEPS = 16;

export const INSTRUMENTS: InstrumentName[] = ["kick", "snare", "hihat", "clap"];

const emptyRow = (): boolean[] => Array.from({ length: STEPS }, () => false);

export function emptyPattern(): Pattern {
  const tracks = {} as Tracks;
  for (const name of INSTRUMENTS) tracks[name] = emptyRow();
  return { tracks };
}

export function normalize(raw: { tracks: Partial<Record<string, boolean[]>> }): Pattern {
  const pattern = emptyPattern();
  for (const name of INSTRUMENTS) {
    const row = raw.tracks[name];
    if (row && row.length === STEPS) pattern.tracks[name] = [...row];
  }
  return pattern;
}

export function toggleStep(pattern: Pattern, name: InstrumentName, step: number): Pattern {
  const row = pattern.tracks[name].map((on, i) => (i === step ? !on : on));
  return { tracks: { ...pattern.tracks, [name]: row } };
}
