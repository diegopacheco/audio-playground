import type { ApiError, Note, Preset, RenderRequest } from "./types";

async function failure(res: Response): Promise<Error> {
  const body = (await res.json().catch(() => ({ error: res.statusText }))) as ApiError;
  return new Error(body.error);
}

export async function listPresets(): Promise<Preset[]> {
  const res = await fetch("/api/presets");
  if (!res.ok) throw await failure(res);
  return (await res.json()) as Preset[];
}

export async function savePreset(preset: Preset): Promise<Preset> {
  const res = await fetch("/api/presets", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(preset),
  });
  if (!res.ok) throw await failure(res);
  return (await res.json()) as Preset;
}

export async function deletePreset(name: string): Promise<void> {
  const res = await fetch(`/api/presets/${encodeURIComponent(name)}`, { method: "DELETE" });
  if (!res.ok) throw await failure(res);
}

export async function renderPhrase(preset: Preset, notes: Note[]): Promise<Blob> {
  const request: RenderRequest = { preset, notes };
  const res = await fetch("/api/render", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
  if (!res.ok) throw await failure(res);
  return res.blob();
}
