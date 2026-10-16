export type SourceKind = "noise" | "tone" | "clicks" | "voice-like";

export type LiveSourceKind = Exclude<SourceKind, "voice-like">;

export type Method = "binaural" | "hrtf";

export interface Position {
  azimuth: number;
  elevation: number;
  distance: number;
}

export type Trajectory =
  | { type: "circle"; period: number; distance: number }
  | { type: "figure8"; period: number; size: number }
  | { type: "static"; azimuth: number; elevation: number; distance: number }
  | { type: "flyby"; speed: number; closest: number };

export type TrajectoryType = Trajectory["type"];

export interface TrajectoryInfo {
  label: string;
  description: string;
  defaults: Trajectory;
}

export interface CueReport extends Position {
  itd_ms: number;
  ild_db: number;
  ild_probe_hz: number;
}

export interface PathPoint extends CueReport {
  t: number;
}

export interface RenderRequest {
  source: SourceKind;
  trajectory: Trajectory;
  duration: number;
  method: Method;
}

export interface Point {
  x: number;
  y: number;
}

export interface PannerCoordinates {
  x: number;
  y: number;
  z: number;
}
