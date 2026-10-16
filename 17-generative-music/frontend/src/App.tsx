import { useCallback, useEffect, useRef, useState } from "react";
import { exportFile, fetchOptions, generate } from "./api";
import { play, type Playback } from "./audio/player";
import { pitchName } from "./audio/roll";
import { Controls } from "./components/Controls";
import { PianoRoll } from "./components/PianoRoll";
import { Transport } from "./components/Transport";
import { randomSeed } from "./seed";
import type { ExportKind, GenerateRequest, Options, Song } from "./types";

const INITIAL: GenerateRequest = {
  key: "C",
  scale: "major",
  progression: "I-V-vi-IV",
  bars: 8,
  bpm: 110,
  generator: "rule",
  seed: 42,
  order: 2,
};

export function App() {
  const [options, setOptions] = useState<Options | null>(null);
  const [request, setRequest] = useState<GenerateRequest>(INITIAL);
  const [song, setSong] = useState<Song | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [playhead, setPlayhead] = useState<number | null>(null);
  const playback = useRef<Playback | null>(null);
  const frame = useRef<number>(0);

  const stop = useCallback(() => {
    playback.current?.stop();
    playback.current = null;
    cancelAnimationFrame(frame.current);
    setPlayhead(null);
  }, []);

  const run = useCallback(
    async (next: GenerateRequest) => {
      stop();
      setBusy(true);
      setError(null);
      try {
        setSong(await generate(next));
      } catch (e) {
        setError((e as Error).message);
      } finally {
        setBusy(false);
      }
    },
    [stop],
  );

  useEffect(() => {
    fetchOptions()
      .then(setOptions)
      .catch((e: Error) => setError(e.message));
    void run(INITIAL);
  }, [run]);

  const newSeed = () => {
    const next = { ...request, seed: randomSeed() };
    setRequest(next);
    void run(next);
  };

  const start = () => {
    if (!song) return;
    stop();
    const current = play(song, stop);
    playback.current = current;
    const tick = () => {
      const seconds = current.context.currentTime - current.startedAt;
      setPlayhead(Math.max(0, (seconds * song.bpm) / 60));
      frame.current = requestAnimationFrame(tick);
    };
    frame.current = requestAnimationFrame(tick);
  };

  const download = async (kind: ExportKind) => {
    setBusy(true);
    setError(null);
    try {
      const blob = await exportFile(kind, request);
      const url = URL.createObjectURL(blob);
      const link = document.createElement("a");
      link.href = url;
      link.download = `generative-${request.seed}.${kind === "midi" ? "mid" : "wav"}`;
      link.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const lowest = song?.notes.length ? Math.min(...song.notes.map((n) => n.pitch)) : null;
  const highest = song?.notes.length ? Math.max(...song.notes.map((n) => n.pitch)) : null;

  return (
    <main>
      <h1>Generative Music</h1>
      <p className="muted">Markov chains and voice-leading rules write a melody over a Roman numeral progression.</p>
      <section className="panel">
        {options ? <Controls options={options} request={request} onChange={setRequest} /> : <p className="muted">Loading options</p>}
        <Transport
          playing={playhead !== null}
          busy={busy}
          hasSong={song !== null}
          onGenerate={() => void run(request)}
          onNewSeed={newSeed}
          onPlay={start}
          onStop={stop}
          onDownload={(kind) => void download(kind)}
        />
        {error && <p className="error">{error}</p>}
      </section>
      {song && (
        <section className="panel">
          <h2>Piano roll</h2>
          <p className="muted small">
            {song.notes.length} notes, {song.chords.length} bars at {song.bpm} BPM
            {lowest !== null && highest !== null && `, range ${pitchName(lowest)} to ${pitchName(highest)}`}
          </p>
          <PianoRoll song={song} playhead={playhead} />
        </section>
      )}
    </main>
  );
}
