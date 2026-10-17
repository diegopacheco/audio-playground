export type Waveform = "sine" | "square" | "saw" | "triangle";

export interface ToneRequest {
  waveform: Waveform;
  frequency: number;
  duration_ms: number;
  amplitude: number;
  sample_rate: number;
}

export interface WaveformsInfo {
  waveforms: Waveform[];
  sample_rates: number[];
}

export interface HeaderField {
  offset: number;
  size: number;
  name: string;
  value: string;
  meaning: string;
}

export interface WavInfo {
  sampleRate: number;
  channels: number;
  bitsPerSample: number;
  dataSize: number;
  fields: HeaderField[];
}

export interface Tone {
  request: ToneRequest;
  url: string;
  info: WavInfo;
  samples: Int16Array;
}
