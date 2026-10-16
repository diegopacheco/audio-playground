import type { ChangeEvent } from "react";
import type { Clip } from "../types";

interface Props {
  clip: Clip | null;
  onBuiltIn: () => void;
  onUpload: (clip: Clip) => void;
}

export function SourcePanel({ clip, onBuiltIn, onUpload }: Props) {
  const pick = async (e: ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) onUpload({ name: file.name, bytes: await file.arrayBuffer() });
    e.target.value = "";
  };

  return (
    <section className="panel">
      <h2>Source</h2>
      <div className="row">
        <button onClick={onBuiltIn}>Use built-in plucked tones</button>
        <label className="file">
          Upload WAV
          <input type="file" accept=".wav,audio/wav" onChange={pick} />
        </label>
        <span className="muted">{clip ? clip.name : "no source loaded"}</span>
      </div>
    </section>
  );
}
