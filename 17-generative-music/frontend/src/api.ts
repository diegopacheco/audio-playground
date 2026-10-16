import type { ExportKind, GenerateRequest, Options, Song } from "./types";

async function failure(response: Response): Promise<Error> {
  const body = (await response.json().catch(() => null)) as { error?: string } | null;
  return new Error(body?.error ?? `request failed with ${response.status}`);
}

function post(path: string, request: GenerateRequest): Promise<Response> {
  return fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function fetchOptions(): Promise<Options> {
  const response = await fetch("/api/options");
  if (!response.ok) throw await failure(response);
  return (await response.json()) as Options;
}

export async function generate(request: GenerateRequest): Promise<Song> {
  const response = await post("/api/generate", request);
  if (!response.ok) throw await failure(response);
  return (await response.json()) as Song;
}

export async function exportFile(kind: ExportKind, request: GenerateRequest): Promise<Blob> {
  const response = await post(`/api/export/${kind}`, request);
  if (!response.ok) throw await failure(response);
  return response.blob();
}
