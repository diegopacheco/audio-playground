export type Generator = "markov" | "rule";

export interface Note {
  pitch: number;
  start: number;
  duration: number;
  velocity: number;
}

export interface Chord {
  symbol: string;
  root: number;
  pitches: number[];
  start: number;
  duration: number;
}

export interface Song {
  notes: Note[];
  chords: Chord[];
  bpm: number;
  bars: number;
}

export interface GenerateRequest {
  key: string;
  scale: string;
  progression: string;
  bars: number;
  bpm: number;
  generator: Generator;
  seed: number;
  order: number;
}

export interface Options {
  keys: string[];
  scales: string[];
  generators: Generator[];
  orders: number[];
  progressions: string[];
  max_bars: number;
  min_bpm: number;
  max_bpm: number;
}

export type ExportKind = "midi" | "wav";

export interface ScheduledTone {
  time: number;
  frequency: number;
  duration: number;
  gain: number;
  voice: "lead" | "pad";
}

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RollLayout {
  width: number;
  height: number;
  low: number;
  high: number;
  beats: number;
}
