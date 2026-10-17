export type WindowName = "hann" | "hamming" | "rect";

export type SourceKind = "tone" | "mic";

export interface Settings {
  window: WindowName;
  fftSize: number;
  source: SourceKind;
  toneHz: number;
}

export interface SpectrumFrame {
  fft_size: number;
  sample_rate: number;
  bars: number[];
  edges: number[];
  peak_hz: number;
  peak_db: number;
}

export interface SpectrumRequest {
  samples: number[];
  sample_rate: number;
  window: WindowName;
  bars: number;
}

export interface StreamParams {
  sample_rate: number;
  window: WindowName;
  bars: number;
}

export interface ErrorReply {
  error: string;
}

export interface Capture {
  sampleRate: number;
  setToneHz: (hz: number) => void;
  stop: () => Promise<void>;
}

export type Rgb = [number, number, number];
