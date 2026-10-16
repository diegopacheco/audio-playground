import type { EffectSpec, ErrorBody, VoiceSettings } from "./types";

async function failure(response: Response): Promise<Error> {
  const text = await response.text();
  try {
    return new Error((JSON.parse(text) as ErrorBody).error);
  } catch {
    return new Error(text || `request failed with ${response.status}`);
  }
}

export async function fetchEffects(): Promise<EffectSpec[]> {
  const response = await fetch("/api/effects");
  if (!response.ok) {
    throw await failure(response);
  }
  return (await response.json()) as EffectSpec[];
}

export async function fetchTestSignal(rate: number, seconds: number): Promise<Blob> {
  const response = await fetch(`/api/test-signal?rate=${rate}&seconds=${seconds}`);
  if (!response.ok) {
    throw await failure(response);
  }
  return response.blob();
}

export async function processWav(wav: Blob, settings: VoiceSettings): Promise<Blob> {
  const query = new URLSearchParams({ effect: settings.effect, semitones: String(settings.semitones) });
  const response = await fetch(`/api/process?${query}`, {
    method: "POST",
    headers: { "Content-Type": "audio/wav" },
    body: wav,
  });
  if (!response.ok) {
    throw await failure(response);
  }
  return response.blob();
}

export function voiceSocketUrl(origin: string, settings: VoiceSettings, rate: number): string {
  const url = new URL("/api/voice", origin);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  url.searchParams.set("effect", settings.effect);
  url.searchParams.set("semitones", String(settings.semitones));
  url.searchParams.set("rate", String(Math.round(rate)));
  return url.toString();
}
