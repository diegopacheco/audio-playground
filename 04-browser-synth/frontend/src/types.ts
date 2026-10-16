export type Waveform = "sine" | "square" | "sawtooth" | "triangle";

export const WAVEFORMS: readonly Waveform[] = ["sine", "square", "sawtooth", "triangle"];

export interface Adsr {
  attack: number;
  decay: number;
  sustain: number;
  release: number;
}

export interface Preset extends Adsr {
  name: string;
  waveform: Waveform;
  cutoff: number;
  resonance: number;
  gain: number;
}

export interface Note {
  midi: number;
  start: number;
  duration: number;
}

export interface RenderRequest {
  preset: Preset;
  notes: Note[];
}

export interface ApiError {
  error: string;
}

export interface BiquadCoefficients {
  b0: number;
  b1: number;
  b2: number;
  a1: number;
  a2: number;
}
