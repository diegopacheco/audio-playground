import { useEffect, useMemo, useRef, useState } from "react";
import { detectOnce, openStream, type StreamClient } from "./api";
import { AudioEngine } from "./audio/engine";
import { noteFrequency } from "./audio/notes";
import { NoteDisplay } from "./components/NoteDisplay";
import { SettingsPanel } from "./components/SettingsPanel";
import { SourcePanel } from "./components/SourcePanel";
import { TunerGauge } from "./components/TunerGauge";
import type { DetectRequest, DetectResponse, DetectorName, NoteChoice, Source, Waveform } from "./types";

const FRAME_INTERVAL_MS = 50;

export function App() {
  const engine = useRef(new AudioEngine());
  const stream = useRef<StreamClient | null>(null);
  const [source, setSource] = useState<Source>("off");
  const [detector, setDetector] = useState<DetectorName>("yin");
  const [a4, setA4] = useState(440);
  const [note, setNote] = useState<NoteChoice>({ name: "A", octave: 4 });
  const [detune, setDetune] = useState(0);
  const [waveform, setWaveform] = useState<Waveform>("sine");
  const [reading, setReading] = useState<DetectResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const toneFrequency = useMemo(
    () => noteFrequency(note, a4) * Math.pow(2, detune / 1200),
    [note, a4, detune],
  );

  const settings = useRef({ detector, a4 });
  settings.current = { detector, a4 };

  useEffect(() => {
    engine.current.updateTone(toneFrequency, waveform);
  }, [toneFrequency, waveform]);

  useEffect(() => {
    if (source === "off") {
      return;
    }
    const client = openStream(setReading, setError);
    stream.current = client;
    const timer = window.setInterval(() => {
      const request = buildRequest();
      if (request) {
        client.send(request);
      }
    }, FRAME_INTERVAL_MS);
    return () => {
      window.clearInterval(timer);
      client.close();
      stream.current = null;
    };
  }, [source]);

  useEffect(() => () => engine.current.stop(), []);

  function buildRequest(): DetectRequest | null {
    const frame = engine.current.frame();
    if (!frame) {
      return null;
    }
    return { ...frame, ...settings.current };
  }

  async function run(action: () => Promise<void>, next: Source) {
    setError(null);
    try {
      await action();
      setSource(next);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setSource("off");
    }
  }

  async function oneShot() {
    const request = buildRequest();
    if (!request) {
      return;
    }
    try {
      setReading(await detectOnce(request));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  return (
    <main>
      <header>
        <h1>Pitch Tuner</h1>
        <p>YIN and autocorrelation pitch detection running in Rust, streamed over WebSocket.</p>
      </header>
      <section className="tuner">
        <NoteDisplay reading={reading} />
        <TunerGauge cents={reading?.cents ?? null} />
        <p className="status">
          {source === "off" ? "idle" : `listening to ${source}`} · detector {reading?.detector ?? detector}
        </p>
        {error ? <p className="error">{error}</p> : null}
      </section>
      <div className="panels">
        <SourcePanel
          source={source}
          note={note}
          detune={detune}
          waveform={waveform}
          toneFrequency={toneFrequency}
          onMic={() => run(() => engine.current.startMic(), "mic")}
          onTone={() => run(() => engine.current.startTone(toneFrequency, waveform), "tone")}
          onStop={() => {
            engine.current.stop();
            setSource("off");
            setReading(null);
          }}
          onNote={setNote}
          onDetune={setDetune}
          onWaveform={setWaveform}
          onOneShot={oneShot}
        />
        <SettingsPanel detector={detector} a4={a4} onDetector={setDetector} onA4={setA4} />
      </div>
    </main>
  );
}
