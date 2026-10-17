import { useEffect, useRef, useState } from "react";
import { fetchEffects } from "./api";
import { VoiceEngine } from "./audio/engine";
import { SILENCE_DB } from "./audio/metrics";
import { EffectControls } from "./components/EffectControls";
import { LatencyPanel } from "./components/LatencyPanel";
import { LevelMeter } from "./components/LevelMeter";
import { OfflinePanel } from "./components/OfflinePanel";
import { SourceControls } from "./components/SourceControls";
import type { EffectSpec, EngineSnapshot, SessionInfo, SourceKind, VoiceSettings } from "./types";

const SNAPSHOT_INTERVAL_MS = 100;

const EMPTY_SNAPSHOT: EngineSnapshot = {
  inputDb: SILENCE_DB,
  outputDb: SILENCE_DB,
  latency: { blockMs: 0, roundTripMs: 0, algorithmMs: 0, bufferMs: 0, totalMs: 0 },
};

export function App() {
  const [effects, setEffects] = useState<EffectSpec[]>([]);
  const [settings, setSettings] = useState<VoiceSettings>({ effect: "pitch", semitones: 4 });
  const [source, setSource] = useState<SourceKind>("off");
  const [busy, setBusy] = useState(false);
  const [session, setSession] = useState<SessionInfo | null>(null);
  const [snapshot, setSnapshot] = useState<EngineSnapshot>(EMPTY_SNAPSHOT);
  const [error, setError] = useState<string | null>(null);
  const engine = useRef<VoiceEngine | null>(null);

  if (!engine.current) {
    engine.current = new VoiceEngine({ onSession: setSession, onError: setError });
  }

  useEffect(() => {
    fetchEffects()
      .then((list) => {
        setEffects(list);
        const first = list[0];
        if (first) {
          setSettings({ effect: first.name, semitones: first.defaultSemitones });
        }
      })
      .catch((err: unknown) => setError(err instanceof Error ? err.message : String(err)));
    return () => engine.current?.stop();
  }, []);

  useEffect(() => {
    if (source === "off") {
      setSnapshot(EMPTY_SNAPSHOT);
      return;
    }
    const timer = window.setInterval(() => {
      if (engine.current) {
        setSnapshot(engine.current.snapshot());
      }
    }, SNAPSHOT_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [source]);

  function changeSettings(next: VoiceSettings) {
    setSettings(next);
    engine.current?.update(next);
  }

  async function start(kind: "mic" | "test") {
    setBusy(true);
    setError(null);
    try {
      await engine.current?.start(kind, settings);
      setSource(kind);
    } catch (err) {
      engine.current?.stop();
      setSource("off");
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  function stop() {
    engine.current?.stop();
    setSource("off");
    setSession(null);
  }

  return (
    <main>
      <header>
        <h1>Voice Changer</h1>
        <p>Phase vocoder pitch shift, robot and whisper voices, streamed block by block through a Rust backend.</p>
      </header>
      {error && (
        <div className="error" role="alert">
          {error}
        </div>
      )}
      <div className="grid">
        <SourceControls source={source} busy={busy} onStart={start} onStop={stop} />
        <EffectControls effects={effects} settings={settings} onChange={changeSettings} />
        <section className="card">
          <h2>Levels</h2>
          <LevelMeter label="Input" db={snapshot.inputDb} />
          <LevelMeter label="Output" db={snapshot.outputDb} />
          <p className="hint">
            {session
              ? `Session: ${session.effect}, ${session.semitones} st, ${session.rate} Hz`
              : "No active stream"}
          </p>
        </section>
        <LatencyPanel latency={snapshot.latency} />
      </div>
      <OfflinePanel settings={settings} onError={setError} />
    </main>
  );
}
