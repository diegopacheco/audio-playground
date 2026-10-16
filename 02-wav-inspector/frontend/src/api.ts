import type { AsciiSize, Report } from "./types";

export async function inspectWav(file: Blob, size: AsciiSize): Promise<Report> {
  const query = new URLSearchParams({ width: String(size.width), height: String(size.height) });
  const res = await fetch(`/api/inspect?${query}`, {
    method: "POST",
    headers: { "Content-Type": "audio/wav" },
    body: file,
  });
  const body: unknown = await res.json().catch(() => ({ error: `HTTP ${res.status}` }));
  if (!res.ok) {
    const message = (body as { error?: string }).error ?? `HTTP ${res.status}`;
    throw new Error(message);
  }
  return body as Report;
}
