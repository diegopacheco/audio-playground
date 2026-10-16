import { useEffect, useState } from "react";
import { getDevices, getStats, measure, startEngine, stopEngine, streamStats } from "./api";
import { DevicePicker } from "./components/DevicePicker";
import { EngineControls } from "./components/EngineControls";
import { LatencyChart } from "./components/LatencyChart";
import { RingGauge } from "./components/RingGauge";
import { XrunPanel } from "./components/XrunPanel";
import { appendHistory } from "./latency";
import type { DeviceList, EngineStatus, Mode, ProbeName } from "./types";

const HISTORY_LIMIT = 30;

export default function App() {
  const [devices, setDevices] = useState<DeviceList | null>(null);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [mode, setMode] = useState<Mode>("measure");
  const [bufferFrames, setBufferFrames] = useState(256);
  const [probe, setProbe] = useState<ProbeName>("correlation");
  const [status, setStatus] = useState<EngineStatus | null>(null);
  const [history, setHistory] = useState<number[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    getDevices()
      .then((d) => {
        setDevices(d);
        setInput(d.default_input ?? d.inputs[0]?.id ?? "");
        setOutput(d.default_output ?? d.outputs[0]?.id ?? "");
      })
      .catch((e: Error) => setError(e.message));
    getStats().then(setStatus).catch(() => undefined);
    return streamStats(setStatus);
  }, []);

  const act = async (fn: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const onStart = () =>
    act(async () => {
      setStatus(await startEngine({ mode, buffer_frames: bufferFrames, input, output, probe }));
    });
  const onStop = () => act(async () => setStatus(await stopEngine()));
  const onMeasure = () =>
    act(async () => {
      const r = await measure();
      setHistory((h) => appendHistory(h, r.round_trip_ms, HISTORY_LIMIT));
    });

  const running = status?.running ?? false;
  const stream = status?.stream;

  return (
    <main>
      <header>
        <h1>Low Latency Audio</h1>
        <p className="muted">
          cpal input callback, lock-free SPSC ring, cpal output callback. Measure the round trip with a
          probe signal.
        </p>
      </header>

      <section className="card">
        <div className="pickers">
          <DevicePicker
            label="Input device"
            devices={devices?.inputs ?? []}
            value={input}
            disabled={running}
            onChange={setInput}
          />
          <DevicePicker
            label="Output device"
            devices={devices?.outputs ?? []}
            value={output}
            disabled={running}
            onChange={setOutput}
          />
        </div>
        <EngineControls
          mode={mode}
          bufferFrames={bufferFrames}
          probe={probe}
          running={running}
          busy={busy}
          onMode={setMode}
          onBuffer={setBufferFrames}
          onProbe={setProbe}
          onStart={onStart}
          onStop={onStop}
          onMeasure={onMeasure}
        />
        <div className="engine-line">
          <span className={`badge ${running ? "healthy" : "idle"}`}>{running ? "running" : "stopped"}</span>
          {stream && (
            <span className="muted">
              {stream.backend} backend, {stream.input_device} to {stream.output_device}, {status?.mode} mode,{" "}
              {status?.probe} probe
            </span>
          )}
        </div>
        {status?.fallback_reason && (
          <p className="warn">Fell back to fake loopback: {status.fallback_reason}</p>
        )}
        {error && <p className="error">{error}</p>}
      </section>

      <div className="grid-2">
        <RingGauge status={status} />
        <XrunPanel status={status} />
      </div>
      <LatencyChart values={history} />
    </main>
  );
}
