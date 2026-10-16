export interface Segment {
  start: number;
  end: number;
  text: string;
}

export interface Transcript {
  text: string;
  segments: Segment[];
  duration: number;
}

export type SummaryMode = "summary" | "action_items";

export interface SummaryResult {
  summary: string;
  action_items: string[];
}

export interface Health {
  whisper: boolean;
  model: boolean;
  claude_key: boolean;
}

export interface ApiErrorBody {
  error: string;
}
