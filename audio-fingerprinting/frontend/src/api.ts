import type { ClipRequest, ErrorResponse, Identification, TrackInfo } from "./types";
import { float32ToBytes } from "./audio/pcm";

async function parse<T>(response: Response): Promise<T> {
  const body = (await response.json()) as T | ErrorResponse;
  if (!response.ok) {
    throw new Error((body as ErrorResponse).error ?? `request failed ${response.status}`);
  }
  return body as T;
}

export async function listLibrary(): Promise<TrackInfo[]> {
  return parse<TrackInfo[]>(await fetch("/api/library"));
}

export async function addTrack(name: string, wav: ArrayBuffer): Promise<TrackInfo> {
  const response = await fetch(`/api/library?name=${encodeURIComponent(name)}`, {
    method: "POST",
    headers: { "Content-Type": "audio/wav" },
    body: wav,
  });
  return parse<TrackInfo>(response);
}

export async function identifyWav(wav: ArrayBuffer): Promise<Identification> {
  const response = await fetch("/api/identify", {
    method: "POST",
    headers: { "Content-Type": "audio/wav" },
    body: wav,
  });
  return parse<Identification>(response);
}

export async function identifyPcm(samples: Float32Array, rate: number): Promise<Identification> {
  const response = await fetch(`/api/identify?rate=${rate}`, {
    method: "POST",
    headers: { "Content-Type": "application/octet-stream" },
    body: float32ToBytes(samples),
  });
  return parse<Identification>(response);
}

export function clipUrl(clip: ClipRequest): string {
  const params = new URLSearchParams({
    track: String(clip.track),
    start: clip.start.toFixed(2),
    len: String(clip.len),
    noise: String(clip.noise),
  });
  return `/api/clip?${params.toString()}`;
}

export async function fetchClip(clip: ClipRequest): Promise<ArrayBuffer> {
  const response = await fetch(clipUrl(clip));
  if (!response.ok) {
    await parse<never>(response);
  }
  return response.arrayBuffer();
}
