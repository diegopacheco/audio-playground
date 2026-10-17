import type { ApiErrorBody } from "../types";

export function formatTimestamp(seconds: number): string {
  const totalMs = Math.max(0, Math.round(seconds * 1000));
  const minutes = Math.floor(totalMs / 60000);
  const secs = Math.floor((totalMs % 60000) / 1000);
  const ms = totalMs % 1000;
  return `${String(minutes).padStart(2, "0")}:${String(secs).padStart(2, "0")}.${String(ms).padStart(3, "0")}`;
}

function isApiError(body: unknown): body is ApiErrorBody {
  return typeof body === "object" && body !== null && typeof (body as ApiErrorBody).error === "string";
}

export function errorMessage(status: number, body: unknown): string {
  if (isApiError(body)) return `${status}: ${body.error}`;
  return `${status}: request failed`;
}

export function pickRecorderMime(isSupported: (mime: string) => boolean): string {
  const candidates = ["audio/webm;codecs=opus", "audio/webm", "audio/mp4"];
  return candidates.find(isSupported) ?? "";
}
