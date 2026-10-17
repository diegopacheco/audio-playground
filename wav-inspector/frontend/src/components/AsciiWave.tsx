import type { AsciiSize } from "../types";

interface Props {
  ascii: string;
  size: AsciiSize;
  onSize: (size: AsciiSize) => void;
}

export function AsciiWave({ ascii, size, onSize }: Props) {
  return (
    <section className="card">
      <div className="row">
        <h2>ASCII waveform</h2>
        <label>
          width
          <input
            type="number"
            min={8}
            max={400}
            value={size.width}
            onChange={(e) => onSize({ ...size, width: Number(e.target.value) })}
          />
        </label>
        <label>
          height
          <input
            type="number"
            min={3}
            max={100}
            value={size.height}
            onChange={(e) => onSize({ ...size, height: Number(e.target.value) })}
          />
        </label>
      </div>
      <pre className="ascii">{ascii}</pre>
    </section>
  );
}
