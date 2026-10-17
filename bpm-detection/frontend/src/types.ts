export type DetectorKind = "flux" | "energy";

export interface Analysis {
  bpm: number;
  confidence: number;
  detector: DetectorKind;
  frame_rate: number;
  lag: number;
  duration_sec: number;
  onset_envelope: number[];
  beats_sec: number[];
  autocorr: number[];
}

export interface ApiError {
  error: string;
}

export interface Track {
  name: string;
  wav: Blob;
}

export interface Point {
  x: number;
  y: number;
}
