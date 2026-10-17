import { useState } from "react";

interface Props {
  names: string[];
  bars: number;
  onLoad: (name: string) => void;
  onSave: (name: string) => void;
  onBars: (bars: number) => void;
  onExport: () => void;
  onClear: () => void;
}

export function PatternBar({ names, bars, onLoad, onSave, onBars, onExport, onClear }: Props) {
  const [name, setName] = useState("my-beat");
  return (
    <div className="patterns">
      <select value="" onChange={(e) => e.target.value && onLoad(e.target.value)}>
        <option value="">Load pattern...</option>
        {names.map((n) => (
          <option key={n} value={n}>
            {n}
          </option>
        ))}
      </select>
      <input value={name} onChange={(e) => setName(e.target.value)} placeholder="pattern name" />
      <button onClick={() => onSave(name)}>Save</button>
      <button onClick={onClear}>Clear</button>
      <label>
        Bars
        <input type="number" min={1} max={16} value={bars} onChange={(e) => onBars(Number(e.target.value))} />
      </label>
      <button className="primary" onClick={onExport}>
        Export WAV
      </button>
    </div>
  );
}
