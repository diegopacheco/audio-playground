import { useCallback, useRef, useState } from "react";
import { listenUrl } from "./api";
import type { ByteEvent, FrameEvent, ListenEvent, ModemSettings } from "./types";

const MAX_FRAMES = 20;

export interface Receiver {
  active: boolean;
  status: string;
  bytes: ByteEvent[];
  frames: FrameEvent[];
  open: (sampleRate: number) => Promise<void>;
  send: (frame: Float32Array) => void;
  close: () => void;
  clear: () => void;
}

export function useReceiver(settings: ModemSettings): Receiver {
  const socket = useRef<WebSocket | null>(null);
  const [active, setActive] = useState(false);
  const [status, setStatus] = useState("idle");
  const [bytes, setBytes] = useState<ByteEvent[]>([]);
  const [frames, setFrames] = useState<FrameEvent[]>([]);

  const handle = useCallback((event: ListenEvent) => {
    switch (event.type) {
      case "locked":
        setBytes([]);
        setStatus(`preamble locked at sample ${event.sample}`);
        break;
      case "byte":
        setBytes((prev) => [...prev, event]);
        break;
      case "frame":
        setFrames((prev) => [event, ...prev].slice(0, MAX_FRAMES));
        setStatus(event.crc_ok ? "frame received, CRC ok" : "frame received, CRC FAILED");
        break;
      case "lost":
        setStatus(`lost: ${event.reason}`);
        break;
    }
  }, []);

  const close = useCallback(() => {
    socket.current?.close();
    socket.current = null;
    setActive(false);
  }, []);

  const open = useCallback(
    (sampleRate: number) =>
      new Promise<void>((resolve, reject) => {
        socket.current?.close();
        const ws = new WebSocket(listenUrl(settings, sampleRate));
        ws.binaryType = "arraybuffer";
        ws.onopen = () => {
          setActive(true);
          setStatus("searching for preamble");
          resolve();
        };
        ws.onerror = () => reject(new Error("listen websocket failed"));
        ws.onclose = () => setActive(false);
        ws.onmessage = (m: MessageEvent<string>) => handle(JSON.parse(m.data) as ListenEvent);
        socket.current = ws;
      }),
    [settings, handle],
  );

  const send = useCallback((frame: Float32Array) => {
    const ws = socket.current;
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(frame.slice());
  }, []);

  const clear = useCallback(() => {
    setBytes([]);
    setFrames([]);
    setStatus("idle");
  }, []);

  return { active, status, bytes, frames, open, send, close, clear };
}
