export type Mode = "passthrough" | "measure";
export type ProbeName = "correlation" | "threshold";

export interface DeviceInfo {
  id: string;
  name: string;
  backend: string;
}

export interface DeviceList {
  inputs: DeviceInfo[];
  outputs: DeviceInfo[];
  default_input: string | null;
  default_output: string | null;
}

export interface StartRequest {
  mode: Mode;
  buffer_frames: number;
  input: string | null;
  output: string | null;
  probe: ProbeName;
}

export interface StreamInfo {
  backend: string;
  sample_rate: number;
  buffer_frames: number;
  input_device: string;
  output_device: string;
  input_channels: number;
  output_channels: number;
}

export interface CallbackSnapshot {
  count: number;
  last_us: number;
  max_us: number;
  avg_us: number;
}

export interface CounterSnapshot {
  underruns: number;
  overruns: number;
  ring_fill: number;
  input_callbacks: CallbackSnapshot;
  output_callbacks: CallbackSnapshot;
}

export interface MeasureResult {
  round_trip_ms: number;
  samples: number;
  sample_rate: number;
  buffer_frames: number;
}

export interface EngineStatus {
  running: boolean;
  mode: Mode | null;
  probe: string | null;
  stream: StreamInfo | null;
  buffer_ms: number | null;
  ring_capacity: number;
  ring_fill_ratio: number;
  counters: CounterSnapshot | null;
  fallback_reason: string | null;
  last_measure: MeasureResult | null;
}

export interface LatencySummary {
  count: number;
  min: number;
  max: number;
  mean: number;
  jitter: number;
}

export type FillLevel = "starving" | "healthy" | "flooding";
