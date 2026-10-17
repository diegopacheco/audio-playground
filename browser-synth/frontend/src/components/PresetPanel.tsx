import { useState } from "react";
import type { Preset } from "../types";

interface Props {
  presets: Preset[];
  current: Preset;
  onLoad: (preset: Preset) => void;
  onSave: (preset: Preset) => void;
  onDelete: (name: string) => void;
}

export function PresetPanel({ presets, current, onLoad, onSave, onDelete }: Props) {
  const [name, setName] = useState(current.name);
  return (
    <section className="panel presets">
      <h2>Presets</h2>
      <ul>
        {presets.map((p) => (
          <li key={p.name} className={p.name === current.name ? "active" : ""}>
            <button
              className="load"
              onClick={() => {
                onLoad(p);
                setName(p.name);
              }}
            >
              {p.name}
              <small>{p.waveform}</small>
            </button>
            <button className="remove" aria-label={`delete ${p.name}`} onClick={() => onDelete(p.name)}>
              x
            </button>
          </li>
        ))}
      </ul>
      <div className="save">
        <input value={name} onChange={(e) => setName(e.target.value)} placeholder="preset name" />
        <button onClick={() => onSave({ ...current, name })}>Save</button>
      </div>
    </section>
  );
}
