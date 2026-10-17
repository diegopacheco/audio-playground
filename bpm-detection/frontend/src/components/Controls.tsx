import { useState } from "react";
import type { DetectorKind } from "../types";

interface Props {
  busy: boolean;
  detector: DetectorKind;
  onDetector: (detector: DetectorKind) => void;
  onUpload: (file: File) => void;
  onSynthetic: (bpm: number) => void;
}

const PRESETS = [90, 120, 128, 174];

export function Controls({ busy, detector, onDetector, onUpload, onSynthetic }: Props) {
  const [bpm, setBpm] = useState(120);
  return (
    <section className="card controls">
      <div className="control">
        <label className="label">Upload WAV</label>
        <input
          type="file"
          accept=".wav,audio/wav"
          disabled={busy}
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) onUpload(file);
          }}
        />
      </div>
      <div className="control">
        <label className="label">Synthetic drum loop</label>
        <div className="row">
          <input
            type="number"
            min={40}
            max={300}
            value={bpm}
            onChange={(e) => setBpm(Number(e.target.value))}
          />
          <span className="muted">BPM</span>
          <button disabled={busy} onClick={() => onSynthetic(bpm)}>
            Generate
          </button>
        </div>
        <div className="row presets">
          {PRESETS.map((p) => (
            <button
              key={p}
              className="chip"
              disabled={busy}
              onClick={() => {
                setBpm(p);
                onSynthetic(p);
              }}
            >
              {p}
            </button>
          ))}
        </div>
      </div>
      <div className="control">
        <label className="label">Onset detector</label>
        <select
          value={detector}
          disabled={busy}
          onChange={(e) => onDetector(e.target.value as DetectorKind)}
        >
          <option value="flux">Spectral flux (FFT)</option>
          <option value="energy">Energy difference</option>
        </select>
      </div>
    </section>
  );
}
