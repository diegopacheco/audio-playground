import { useState } from "react";
import type { ClipRequest, TrackInfo } from "../types";
import { randomClip } from "../audio/pcm";

interface Props {
  tracks: TrackInfo[];
  busy: boolean;
  onFile: (file: File) => void;
  onRecord: (seconds: number) => void;
  onClip: (clip: ClipRequest) => void;
}

export function QueryPanel({ tracks, busy, onFile, onRecord, onClip }: Props) {
  const [seconds, setSeconds] = useState(6);
  const [noise, setNoise] = useState(0.5);
  const [length, setLength] = useState(5);

  const grab = () => {
    const clip = randomClip(tracks, length, noise, Math.random);
    if (clip) {
      onClip(clip);
    }
  };

  return (
    <section className="card">
      <h2>Identify</h2>
      <div className="query-row">
        <label className="label">Upload WAV</label>
        <input
          type="file"
          accept=".wav,audio/wav"
          disabled={busy}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) {
              onFile(file);
            }
            e.target.value = "";
          }}
        />
      </div>
      <div className="query-row">
        <label className="label">Microphone</label>
        <select value={seconds} onChange={(e) => setSeconds(Number(e.target.value))}>
          {[4, 6, 8, 10].map((s) => (
            <option key={s} value={s}>
              {s} seconds
            </option>
          ))}
        </select>
        <button disabled={busy} onClick={() => onRecord(seconds)}>
          Record and identify
        </button>
      </div>
      <div className="query-row">
        <label className="label">Random clip</label>
        <span className="muted">len {length}s</span>
        <input
          type="range"
          min={2}
          max={10}
          step={1}
          value={length}
          onChange={(e) => setLength(Number(e.target.value))}
        />
        <span className="muted">noise {noise.toFixed(1)}</span>
        <input
          type="range"
          min={0}
          max={3}
          step={0.1}
          value={noise}
          onChange={(e) => setNoise(Number(e.target.value))}
        />
        <button disabled={busy || tracks.length === 0} onClick={grab}>
          Grab noisy clip
        </button>
      </div>
      <p className="hint">Noise is white noise RMS relative to the clip RMS (1.0 = 0 dB SNR).</p>
    </section>
  );
}
