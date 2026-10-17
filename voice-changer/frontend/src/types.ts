export interface EffectSpec {
  name: string;
  label: string;
  minSemitones: number;
  maxSemitones: number;
  defaultSemitones: number;
}

export interface VoiceSettings {
  effect: string;
  semitones: number;
}

export interface SessionInfo {
  effect: string;
  semitones: number;
  rate: number;
  latencySamples: number;
}

export interface ErrorBody {
  error: string;
}

export type SourceKind = "off" | "mic" | "test";

export interface LatencyReport {
  blockMs: number;
  roundTripMs: number;
  algorithmMs: number;
  bufferMs: number;
  totalMs: number;
}

export interface EngineSnapshot {
  inputDb: number;
  outputDb: number;
  latency: LatencyReport;
}

export interface EngineCallbacks {
  onSession: (info: SessionInfo) => void;
  onError: (message: string) => void;
}

export interface RenderResult {
  original: Blob;
  processed: Blob;
}
