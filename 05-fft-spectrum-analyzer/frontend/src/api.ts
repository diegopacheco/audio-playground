import type { ErrorReply, SpectrumFrame, SpectrumRequest, StreamParams } from "./types";

const MAX_BUFFERED_BYTES = 1 << 20;

function isError(value: SpectrumFrame | ErrorReply): value is ErrorReply {
  return "error" in value;
}

export async function postSpectrum(req: SpectrumRequest): Promise<SpectrumFrame> {
  const res = await fetch("/api/spectrum", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  const body = (await res.json()) as SpectrumFrame | ErrorReply;
  if (!res.ok || isError(body)) {
    throw new Error(isError(body) ? body.error : `HTTP ${res.status}`);
  }
  return body;
}

export function streamUrl(params: StreamParams, location: { protocol: string; host: string }): string {
  const proto = location.protocol === "https:" ? "wss:" : "ws:";
  const query = new URLSearchParams({
    sample_rate: String(params.sample_rate),
    window: params.window,
    bars: String(params.bars),
  });
  return `${proto}//${location.host}/api/stream?${query.toString()}`;
}

export function openStream(
  params: StreamParams,
  onFrame: (frame: SpectrumFrame) => void,
  onError: (message: string) => void,
): WebSocket {
  const ws = new WebSocket(streamUrl(params, window.location));
  ws.binaryType = "arraybuffer";
  ws.onmessage = (event: MessageEvent<string>) => {
    const body = JSON.parse(event.data) as SpectrumFrame | ErrorReply;
    if (isError(body)) onError(body.error);
    else onFrame(body);
  };
  ws.onerror = () => onError("stream connection failed");
  return ws;
}

export function sendFrame(ws: WebSocket, frame: Float32Array): boolean {
  if (ws.readyState !== WebSocket.OPEN || ws.bufferedAmount > MAX_BUFFERED_BYTES) return false;
  ws.send(frame);
  return true;
}
