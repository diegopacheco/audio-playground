export interface TrackInfo {
  id: number;
  name: string;
  duration_sec: number;
  hashes: number;
  builtin: boolean;
}

export interface MatchedTrack {
  track_id: number;
  name: string;
}

export interface Candidate {
  track_id: number;
  name: string;
  score: number;
  offset_sec: number;
}

export interface PeakPoint {
  t: number;
  hz: number;
}

export interface HistogramBin {
  offset_sec: number;
  count: number;
}

export interface Identification {
  match: MatchedTrack | null;
  score: number;
  offset_sec: number;
  candidates: Candidate[];
  peaks: PeakPoint[];
  histogram: HistogramBin[];
  duration_sec: number;
  hash_count: number;
  min_score: number;
}

export interface ClipRequest {
  track: number;
  start: number;
  len: number;
  noise: number;
}

export interface QueryResult {
  label: string;
  identification: Identification;
}

export interface ErrorResponse {
  error: string;
}
