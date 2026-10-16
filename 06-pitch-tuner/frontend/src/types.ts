export type DetectorName = "yin" | "autocorrelation";

export type Waveform = "sine" | "sawtooth" | "square" | "triangle";

export type Source = "off" | "mic" | "tone";

export interface DetectRequest {
  samples: number[];
  sampleRate: number;
  detector: DetectorName;
  a4: number;
}

export interface DetectResponse {
  frequency: number | null;
  note: string | null;
  octave: number | null;
  cents: number | null;
  clarity: number;
  detector: DetectorName;
}

export interface ErrorResponse {
  error: string;
}

export interface NoteChoice {
  name: string;
  octave: number;
}
