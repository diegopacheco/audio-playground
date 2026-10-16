import type { ClickTrackRequest, DriftReport } from "./types";

async function failure(res: Response): Promise<Error> {
  const body = (await res.json().catch(() => null)) as { error?: string } | null;
  return new Error(body?.error ?? `request failed with ${res.status}`);
}

export async function fetchClickTrack(req: ClickTrackRequest): Promise<Blob> {
  const res = await fetch("/api/click-track", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) throw await failure(res);
  return res.blob();
}

export async function fetchDrift(bpm: number, beats: number): Promise<DriftReport> {
  const res = await fetch(`/api/drift?bpm=${bpm}&beats=${beats}`);
  if (!res.ok) throw await failure(res);
  return (await res.json()) as DriftReport;
}
