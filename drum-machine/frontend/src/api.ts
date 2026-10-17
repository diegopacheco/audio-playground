import { normalize } from "./pattern";
import type { Pattern, PatternList, RenderRequest } from "./types";

async function check(res: Response): Promise<Response> {
  if (res.ok) return res;
  const body = (await res.json().catch(() => ({ error: res.statusText }))) as { error: string };
  throw new Error(body.error);
}

export async function listPatterns(): Promise<string[]> {
  const res = await check(await fetch("/api/patterns"));
  return ((await res.json()) as PatternList).names;
}

export async function loadPattern(name: string): Promise<Pattern> {
  const res = await check(await fetch(`/api/patterns/${encodeURIComponent(name)}`));
  return normalize((await res.json()) as Pattern);
}

export async function savePattern(name: string, pattern: Pattern): Promise<void> {
  await check(
    await fetch(`/api/patterns/${encodeURIComponent(name)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(pattern),
    }),
  );
}

export async function renderWav(req: RenderRequest): Promise<Blob> {
  const res = await check(
    await fetch("/api/render", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(req),
    }),
  );
  return res.blob();
}
