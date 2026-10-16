export type Method = "lsb" | "ultrasonic";

export type Tab = "encode" | "decode";

export interface MethodConfig {
  method: Method;
  bits: number;
}

export interface Capacity {
  method: Method;
  bits: number;
  bytes: number;
  samples: number;
  sampleRate: number;
  seconds: number;
}

export interface EncodeResult {
  wav: Blob;
  snrDb: string;
  maxDiff: number;
  capacity: number;
  payloadBytes: number;
}

export interface DecodeResult {
  message: string;
  bytes: number;
}

export interface PcmAudio {
  sampleRate: number;
  samples: Int16Array;
}

export interface DiffStats {
  maxDiff: number;
  changed: number;
}

export interface Column {
  min: number;
  max: number;
}
