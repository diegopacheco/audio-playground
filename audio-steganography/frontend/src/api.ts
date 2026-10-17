import type { Capacity, DecodeResult, EncodeResult, MethodConfig } from "./types";

async function failure(res: Response): Promise<Error> {
  const body = (await res.json().catch(() => null)) as { error?: string } | null;
  return new Error(body?.error ?? `request failed with ${res.status}`);
}

function query(config: MethodConfig): string {
  return `method=${config.method}&bits=${config.bits}`;
}

export async function fetchCarrier(): Promise<Blob> {
  const res = await fetch("/api/carrier");
  if (!res.ok) throw await failure(res);
  return res.blob();
}

export async function fetchCapacity(config: MethodConfig, wav: Blob | null): Promise<Capacity> {
  const res = wav
    ? await fetch(`/api/capacity?${query(config)}`, { method: "POST", body: wav })
    : await fetch(`/api/capacity?${query(config)}`);
  if (!res.ok) throw await failure(res);
  return (await res.json()) as Capacity;
}

function toBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(new Error("could not read file"));
    reader.readAsDataURL(blob);
  });
}

export async function encode(
  config: MethodConfig,
  message: string,
  wav: Blob | null,
): Promise<EncodeResult> {
  const res = await fetch("/api/encode", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      message,
      method: config.method,
      bits: config.bits,
      wav: wav ? await toBase64(wav) : null,
    }),
  });
  if (!res.ok) throw await failure(res);
  return {
    wav: await res.blob(),
    snrDb: res.headers.get("x-snr-db") ?? "",
    maxDiff: Number(res.headers.get("x-max-diff")),
    capacity: Number(res.headers.get("x-capacity")),
    payloadBytes: Number(res.headers.get("x-payload-bytes")),
  };
}

export async function decode(config: MethodConfig, wav: Blob): Promise<DecodeResult> {
  const res = await fetch(`/api/decode?${query(config)}`, { method: "POST", body: wav });
  if (!res.ok) throw await failure(res);
  return (await res.json()) as DecodeResult;
}
