import { encodeBase64 } from "./audio/base64";
import type { EffectConfig, EffectSpec } from "./types";

async function check(response: Response): Promise<Response> {
  if (response.ok) return response;
  const body = (await response.json().catch(() => null)) as { error?: string } | null;
  throw new Error(body?.error ?? `request failed with ${response.status}`);
}

export async function fetchEffects(): Promise<EffectSpec[]> {
  const response = await check(await fetch("/api/effects"));
  return (await response.json()) as EffectSpec[];
}

export async function fetchSource(): Promise<ArrayBuffer> {
  const response = await check(await fetch("/api/source"));
  return response.arrayBuffer();
}

export async function processAudio(wav: ArrayBuffer, chain: EffectConfig[]): Promise<ArrayBuffer> {
  const response = await check(
    await fetch("/api/process", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ wav_base64: encodeBase64(new Uint8Array(wav)), chain }),
    }),
  );
  return response.arrayBuffer();
}
