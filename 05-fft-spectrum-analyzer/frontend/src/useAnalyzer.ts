import { useCallback, useEffect, useRef, useState } from "react";
import { openStream, postSpectrum, sendFrame } from "./api";
import { startCapture } from "./audio/capture";
import { Framer } from "./audio/framer";
import type { Capture, Settings, SpectrumFrame } from "./types";

export const BAR_COUNT = 64;

export interface Analyzer {
  running: boolean;
  frame: SpectrumFrame | null;
  snapshot: SpectrumFrame | null;
  error: string | null;
  start: () => Promise<void>;
  stop: () => Promise<void>;
  takeSnapshot: () => Promise<void>;
}

function message(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}

export function useAnalyzer(settings: Settings): Analyzer {
  const [running, setRunning] = useState(false);
  const [frame, setFrame] = useState<SpectrumFrame | null>(null);
  const [snapshot, setSnapshot] = useState<SpectrumFrame | null>(null);
  const [error, setError] = useState<string | null>(null);
  const capture = useRef<Capture | null>(null);
  const socket = useRef<WebSocket | null>(null);
  const lastFrame = useRef<Float32Array | null>(null);
  const generation = useRef(0);
  const active = useRef(false);

  const teardown = useCallback(async () => {
    generation.current += 1;
    const ws = socket.current;
    socket.current = null;
    if (ws) {
      ws.onmessage = null;
      ws.onerror = null;
      ws.close();
    }
    const c = capture.current;
    capture.current = null;
    if (c) await c.stop();
  }, []);

  const launch = useCallback(async () => {
    await teardown();
    const gen = generation.current;
    setError(null);
    const framer = new Framer(settings.fftSize);
    try {
      const c = await startCapture(settings.source, settings.toneHz, (chunk) => {
        for (const f of framer.push(chunk)) {
          lastFrame.current = f;
          if (socket.current) sendFrame(socket.current, f);
        }
      });
      if (gen !== generation.current) {
        await c.stop();
        return;
      }
      capture.current = c;
      socket.current = openStream(
        { sample_rate: c.sampleRate, window: settings.window, bars: BAR_COUNT },
        setFrame,
        setError,
      );
      setRunning(true);
    } catch (e) {
      if (gen !== generation.current) return;
      active.current = false;
      setError(message(e));
      setRunning(false);
      await teardown();
    }
  }, [settings.fftSize, settings.source, settings.window, settings.toneHz, teardown]);

  const start = useCallback(async () => {
    active.current = true;
    await launch();
  }, [launch]);

  const stop = useCallback(async () => {
    active.current = false;
    setRunning(false);
    await teardown();
  }, [teardown]);

  const takeSnapshot = useCallback(async () => {
    const f = lastFrame.current;
    const c = capture.current;
    if (!f || !c) return;
    try {
      const result = await postSpectrum({
        samples: Array.from(f),
        sample_rate: c.sampleRate,
        window: settings.window,
        bars: BAR_COUNT,
      });
      setSnapshot(result);
    } catch (e) {
      setError(message(e));
    }
  }, [settings.window]);

  useEffect(() => {
    capture.current?.setToneHz(settings.toneHz);
  }, [settings.toneHz]);

  const launchRef = useRef(launch);
  useEffect(() => {
    launchRef.current = launch;
  }, [launch]);

  useEffect(() => {
    if (active.current) void launchRef.current();
  }, [settings.fftSize, settings.source, settings.window]);

  useEffect(() => () => void teardown(), [teardown]);

  return { running, frame, snapshot, error, start, stop, takeSnapshot };
}
