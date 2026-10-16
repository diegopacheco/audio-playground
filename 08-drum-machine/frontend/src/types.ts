export type InstrumentName = "kick" | "snare" | "hihat" | "clap";

export type Tracks = Record<InstrumentName, boolean[]>;

export interface Pattern {
  tracks: Tracks;
}

export interface PatternList {
  names: string[];
}

export interface RenderRequest {
  pattern: Pattern;
  bpm: number;
  bars: number;
  swing: number;
}

export interface ScheduledStep {
  step: number;
  time: number;
}

export interface Cursor {
  nextStep: number;
  nextBaseTime: number;
}

export interface ScheduleWindow {
  steps: ScheduledStep[];
  cursor: Cursor;
}

export interface PlaybackSettings {
  pattern: Pattern;
  bpm: number;
  swing: number;
}

export type Voice = (ctx: AudioContext, out: AudioNode, time: number) => void;
