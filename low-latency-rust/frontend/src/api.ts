import type { DeviceList, EngineStatus, MeasureResult, StartRequest } from "./types";

async function call<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  const body = await res.json();
  if (!res.ok) throw new Error(body.error ?? `HTTP ${res.status}`);
  return body as T;
}

function post<T>(path: string, payload?: unknown): Promise<T> {
  return call<T>(path, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: payload === undefined ? undefined : JSON.stringify(payload),
  });
}

export const getDevices = () => call<DeviceList>("/api/devices");
export const getStats = () => call<EngineStatus>("/api/stats");
export const startEngine = (req: StartRequest) => post<EngineStatus>("/api/start", req);
export const stopEngine = () => post<EngineStatus>("/api/stop");
export const measure = () => post<MeasureResult>("/api/measure");

export function streamStats(onStatus: (s: EngineStatus) => void): () => void {
  const proto = location.protocol === "https:" ? "wss" : "ws";
  let socket: WebSocket | null = null;
  let closed = false;
  const connect = () => {
    socket = new WebSocket(`${proto}://${location.host}/api/stats/stream`);
    socket.onmessage = (e) => {
      const data = JSON.parse(e.data);
      if (!("error" in data)) onStatus(data as EngineStatus);
    };
    socket.onclose = () => {
      if (!closed) setTimeout(connect, 1000);
    };
  };
  connect();
  return () => {
    closed = true;
    socket?.close();
  };
}
