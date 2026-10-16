import type { Health, SummaryMode, SummaryResult, Transcript } from "./types";
import { errorMessage } from "./audio/format";

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, init);
  const body: unknown = await response.json().catch(() => null);
  if (!response.ok) throw new Error(errorMessage(response.status, body));
  return body as T;
}

export function fetchHealth(): Promise<Health> {
  return request<Health>("/api/health");
}

export function transcribe(audio: Blob): Promise<Transcript> {
  return request<Transcript>("/api/transcribe", {
    method: "POST",
    headers: { "Content-Type": audio.type || "application/octet-stream" },
    body: audio,
  });
}

export function summarize(text: string, mode: SummaryMode): Promise<SummaryResult> {
  return request<SummaryResult>("/api/summarize", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text, mode }),
  });
}
