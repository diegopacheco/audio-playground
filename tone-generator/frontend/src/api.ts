import type { ToneRequest, WaveformsInfo } from "./types";

async function errorMessage(res: Response): Promise<string> {
  try {
    const body: { error?: string } = await res.json();
    return body.error ?? `request failed with ${res.status}`;
  } catch {
    return `request failed with ${res.status}`;
  }
}

export async function fetchWaveforms(): Promise<WaveformsInfo> {
  const res = await fetch("/api/waveforms");
  if (!res.ok) throw new Error(await errorMessage(res));
  return res.json();
}

export async function fetchTone(req: ToneRequest): Promise<ArrayBuffer> {
  const res = await fetch("/api/tone", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw new Error(await errorMessage(res));
  return res.arrayBuffer();
}
