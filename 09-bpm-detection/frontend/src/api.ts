import type { Analysis, ApiError, DetectorKind } from "./types";

async function failure(response: Response): Promise<Error> {
  try {
    const body = (await response.json()) as ApiError;
    return new Error(body.error);
  } catch {
    return new Error(`request failed with status ${response.status}`);
  }
}

export async function analyzeWav(wav: Blob, detector: DetectorKind): Promise<Analysis> {
  const response = await fetch(`/api/analyze?detector=${detector}`, {
    method: "POST",
    headers: { "Content-Type": "audio/wav" },
    body: wav,
  });
  if (!response.ok) throw await failure(response);
  return (await response.json()) as Analysis;
}

export async function fetchSynthetic(bpm: number): Promise<Blob> {
  const response = await fetch(`/api/synthetic?bpm=${bpm}`);
  if (!response.ok) throw await failure(response);
  return await response.blob();
}
