import { useCallback, useEffect, useState } from "react";
import { analyzeWav, fetchSynthetic } from "./api";
import { AutocorrChart, EnvelopeChart } from "./components/Charts";
import { Controls } from "./components/Controls";
import { Player } from "./components/Player";
import { Summary } from "./components/Summary";
import type { Analysis, DetectorKind, Track } from "./types";

export function App() {
  const [detector, setDetector] = useState<DetectorKind>("flux");
  const [track, setTrack] = useState<Track | null>(null);
  const [url, setUrl] = useState("");
  const [analysis, setAnalysis] = useState<Analysis | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [time, setTime] = useState(0);

  useEffect(() => {
    if (!track) return;
    const objectUrl = URL.createObjectURL(track.wav);
    setUrl(objectUrl);
    return () => URL.revokeObjectURL(objectUrl);
  }, [track]);

  useEffect(() => {
    if (!track) return;
    let cancelled = false;
    setBusy(true);
    setError("");
    analyzeWav(track.wav, detector)
      .then((result) => {
        if (!cancelled) setAnalysis(result);
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setAnalysis(null);
        setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => {
        if (!cancelled) setBusy(false);
      });
    return () => {
      cancelled = true;
    };
  }, [track, detector]);

  const loadSynthetic = async (bpm: number) => {
    setBusy(true);
    setError("");
    try {
      setTrack({ name: `synthetic drum loop at ${bpm} BPM`, wav: await fetchSynthetic(bpm) });
      setTime(0);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  };

  const onTime = useCallback((t: number) => setTime(t), []);

  return (
    <main className="page">
      <header className="title">
        <h1>BPM Detection</h1>
        <p className="muted">spectral flux onsets, autocorrelation tempo, phase aligned beat tracking</p>
      </header>
      <Controls
        busy={busy}
        detector={detector}
        onDetector={setDetector}
        onUpload={(file) => {
          setTrack({ name: file.name, wav: file });
          setTime(0);
        }}
        onSynthetic={loadSynthetic}
      />
      {error && <div className="error">{error}</div>}
      {busy && <div className="muted">analyzing...</div>}
      {track && url && (
        <Player key={url} url={url} name={track.name} beats={analysis?.beats_sec ?? []} onTime={onTime} />
      )}
      {analysis && (
        <>
          <Summary analysis={analysis} />
          <EnvelopeChart analysis={analysis} time={time} />
          <AutocorrChart analysis={analysis} />
        </>
      )}
    </main>
  );
}
