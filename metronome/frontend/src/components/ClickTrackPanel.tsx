import { useEffect, useState } from "react";
import { fetchClickTrack } from "../api";

interface Props {
  bpm: number;
  beatsPerBar: number;
}

export function ClickTrackPanel({ bpm, beatsPerBar }: Props) {
  const [bars, setBars] = useState(4);
  const [accent, setAccent] = useState(true);
  const [url, setUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => () => {
    if (url) URL.revokeObjectURL(url);
  }, [url]);

  const render = async () => {
    setBusy(true);
    setError(null);
    try {
      const blob = await fetchClickTrack({ bpm, beats_per_bar: beatsPerBar, bars, accent });
      setUrl(URL.createObjectURL(blob));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="card">
      <h2>Rust click track</h2>
      <p className="muted">
        Sample-accurate WAV rendered by the backend: every click starts exactly round(60 / bpm x 44100) samples after the previous one.
      </p>
      <div className="row">
        <label>
          Bars
          <input type="number" min={1} max={64} value={bars} onChange={(e) => setBars(Number(e.target.value))} />
        </label>
        <label className="check">
          <input type="checkbox" checked={accent} onChange={(e) => setAccent(e.target.checked)} />
          Accent first beat
        </label>
        <button type="button" onClick={render} disabled={busy}>
          {busy ? "Rendering..." : `Render ${bpm} bpm`}
        </button>
      </div>
      {error && <p className="error">{error}</p>}
      {url && (
        <div className="row">
          <audio controls src={url} />
          <a className="button" href={url} download={`click-${bpm}bpm-${beatsPerBar}x${bars}.wav`}>
            Download WAV
          </a>
        </div>
      )}
    </section>
  );
}
