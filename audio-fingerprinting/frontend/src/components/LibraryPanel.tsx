import { useState } from "react";
import type { TrackInfo } from "../types";

interface Props {
  tracks: TrackInfo[];
  busy: boolean;
  onAdd: (name: string, wav: ArrayBuffer) => void;
}

export function LibraryPanel({ tracks, busy, onAdd }: Props) {
  const [name, setName] = useState("");
  const [file, setFile] = useState<File | null>(null);

  const submit = async () => {
    if (!file) {
      return;
    }
    onAdd(name.trim() || file.name.replace(/\.wav$/i, ""), await file.arrayBuffer());
    setName("");
    setFile(null);
  };

  return (
    <section className="card">
      <h2>Library</h2>
      <ul className="tracks">
        {tracks.map((t) => (
          <li key={t.id}>
            <span className="track-name">{t.name}</span>
            <span className="muted">
              {t.duration_sec.toFixed(1)}s · {t.hashes} hashes{t.builtin ? "" : " · added"}
            </span>
          </li>
        ))}
      </ul>
      <div className="add-track">
        <input
          type="text"
          placeholder="Track name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <input
          type="file"
          accept=".wav,audio/wav"
          onChange={(e) => setFile(e.target.files?.[0] ?? null)}
        />
        <button disabled={!file || busy} onClick={submit}>
          Add WAV to library
        </button>
      </div>
    </section>
  );
}
