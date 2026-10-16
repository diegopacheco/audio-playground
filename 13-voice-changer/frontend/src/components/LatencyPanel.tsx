import type { LatencyReport } from "../types";

interface Props {
  latency: LatencyReport;
}

const rows: { key: keyof LatencyReport; label: string }[] = [
  { key: "blockMs", label: "Capture block" },
  { key: "roundTripMs", label: "WebSocket round trip" },
  { key: "algorithmMs", label: "Effect (STFT) latency" },
  { key: "bufferMs", label: "Playback buffer" },
];

export function LatencyPanel({ latency }: Props) {
  return (
    <section className="card">
      <h2>Latency</h2>
      <div className="latency-total">
        <span className="mono">{latency.totalMs.toFixed(0)}</span> ms mic to speaker
      </div>
      <dl className="latency-rows">
        {rows.map((row) => (
          <div key={row.key}>
            <dt>{row.label}</dt>
            <dd className="mono">{latency[row.key].toFixed(1)} ms</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}
