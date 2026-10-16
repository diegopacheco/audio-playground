import type { WavInfo } from "../types";

interface Props {
  info: WavInfo;
  hex: string[];
}

export function HeaderPanel({ info, hex }: Props) {
  return (
    <div className="card">
      <h2>WAV header (44 bytes)</h2>
      <table>
        <thead>
          <tr>
            <th>Offset</th>
            <th>Bytes</th>
            <th>Field</th>
            <th>Hex (little-endian)</th>
            <th>Value</th>
            <th>Meaning</th>
          </tr>
        </thead>
        <tbody>
          {info.fields.map((f) => (
            <tr key={f.offset}>
              <td>{f.offset}</td>
              <td>{f.size}</td>
              <td className="mono">{f.name}</td>
              <td className="mono">{hex.slice(f.offset, f.offset + f.size).join(" ")}</td>
              <td className="mono">{f.value}</td>
              <td className="muted">{f.meaning}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
