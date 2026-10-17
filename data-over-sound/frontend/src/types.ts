export type Scheme = "bfsk" | "4fsk";

export interface ModemSettings {
  scheme: Scheme;
  baud: number;
}

export interface SchemeInfo {
  scheme: Scheme;
  tones: number[];
  bits_per_symbol: number;
  min_baud: number;
  max_baud: number;
}

export interface PcmAudio {
  sampleRate: number;
  samples: Float32Array;
}

export interface DemodResult {
  text: string;
  crc_ok: boolean;
  snr: number;
}

export interface LockedEvent {
  type: "locked";
  sample: number;
}

export interface ByteEvent {
  type: "byte";
  value: number;
  bits: string;
  framing_ok: boolean;
}

export interface FrameEvent {
  type: "frame";
  text: string;
  crc_ok: boolean;
  snr: number;
}

export interface LostEvent {
  type: "lost";
  reason: string;
}

export type ListenEvent = LockedEvent | ByteEvent | FrameEvent | LostEvent;

export type ByteRole = "sync" | "length" | "data" | "crc";

export interface LoggedByte extends ByteEvent {
  index: number;
  role: ByteRole;
}

export type ReceiveSource = "mic" | "loopback";

export interface Spectrogram {
  frames: number;
  bins: number;
  maxHz: number;
  values: Float32Array;
}
