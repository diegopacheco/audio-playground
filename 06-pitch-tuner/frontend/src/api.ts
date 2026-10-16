import type { DetectRequest, DetectResponse, ErrorResponse } from "./types";

export function isError(value: DetectResponse | ErrorResponse): value is ErrorResponse {
  return "error" in value;
}

export async function detectOnce(request: DetectRequest): Promise<DetectResponse> {
  const response = await fetch("/api/detect", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
  const body = (await response.json()) as DetectResponse | ErrorResponse;
  if (isError(body)) {
    throw new Error(body.error);
  }
  return body;
}

export interface StreamClient {
  send: (request: DetectRequest) => void;
  close: () => void;
}

export function openStream(
  onResult: (result: DetectResponse) => void,
  onError: (message: string) => void,
): StreamClient {
  const protocol = window.location.protocol === "https:" ? "wss" : "ws";
  const socket = new WebSocket(`${protocol}://${window.location.host}/api/stream`);
  let waiting = false;
  socket.onmessage = (event: MessageEvent<string>) => {
    waiting = false;
    const body = JSON.parse(event.data) as DetectResponse | ErrorResponse;
    if (isError(body)) {
      onError(body.error);
    } else {
      onResult(body);
    }
  };
  socket.onerror = () => onError("stream connection failed");
  return {
    send: (request) => {
      if (socket.readyState !== WebSocket.OPEN || waiting) {
        return;
      }
      waiting = true;
      socket.send(JSON.stringify(request));
    },
    close: () => socket.close(),
  };
}
