export type Mode = "interval" | "lookahead";

export interface BeatSample {
  index: number;
  errorMs: number;
}

export interface JitterStats {
  count: number;
  meanMs: number;
  maxAbsMs: number;
  stdDevMs: number;
}

export interface ScheduledBeat {
  index: number;
  time: number;
}

export interface ClickTrackRequest {
  bpm: number;
  beats_per_bar: number;
  bars: number;
  accent: boolean;
}

export interface BeatTiming {
  index: number;
  ideal_ms: number;
  naive_ms: number;
  naive_drift_ms: number;
  absolute_ms: number;
  absolute_drift_ms: number;
}

export interface DriftReport {
  bpm: number;
  interval_ms: number;
  naive: string;
  absolute: string;
  naive_max_drift_ms: number;
  absolute_max_drift_ms: number;
  beats: BeatTiming[];
}

export interface ChartSeries {
  name: string;
  color: string;
  values: number[];
}

export type BeatListener = (sample: BeatSample) => void;
