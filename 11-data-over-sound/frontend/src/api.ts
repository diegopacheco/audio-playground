import type { DemodResult, ModemSettings, SchemeInfo } from "./types";

async function errorText(res: Response): Promise<string> {
  try {
    const body = (await res.json()) as { error?: string };
    return body.error ?? res.statusText;
  } catch {
    return res.statusText;
  }
}

export async function fetchSchemes(): Promise<SchemeInfo[]> {
  const res = await fetch("/api/schemes");
  if (!res.ok) throw new Error(await errorText(res));
  return (await res.json()) as SchemeInfo[];
}

export async function modulate(text: string, settings: ModemSettings): Promise<ArrayBuffer> {
  const res = await fetch("/api/modulate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ text, scheme: settings.scheme, baud: settings.baud }),
  });
  if (!res.ok) throw new Error(await errorText(res));
  return res.arrayBuffer();
}

export async function demodulate(
  samples: Float32Array,
  sampleRate: number,
  settings: ModemSettings,
): Promise<DemodResult> {
  const query = new URLSearchParams({
    scheme: settings.scheme,
    baud: String(settings.baud),
    sample_rate: String(sampleRate),
  });
  const res = await fetch(`/api/demodulate?${query}`, {
    method: "POST",
    headers: { "Content-Type": "application/octet-stream" },
    body: new Blob([samples.slice()]),
  });
  if (!res.ok) throw new Error(await errorText(res));
  return (await res.json()) as DemodResult;
}

export function listenUrl(settings: ModemSettings, sampleRate: number): string {
  const protocol = window.location.protocol === "https:" ? "wss" : "ws";
  const query = new URLSearchParams({
    scheme: settings.scheme,
    baud: String(settings.baud),
    sample_rate: String(Math.round(sampleRate)),
  });
  return `${protocol}://${window.location.host}/api/listen?${query}`;
}
