import type { DiffStats, EncodeResult } from "../types";

interface Props {
  result: EncodeResult;
  stats: DiffStats;
  totalSamples: number;
}

export function StatsPanel({ result, stats, totalSamples }: Props) {
  const snr = result.snrDb === "inf" ? "identical" : `${result.snrDb} dB`;
  return (
    <div className="stats">
      <div>
        <span className="value">{snr}</span>
        <span className="label">SNR</span>
      </div>
      <div>
        <span className="value">{result.maxDiff}</span>
        <span className="label">max sample diff</span>
      </div>
      <div>
        <span className="value">{((100 * stats.changed) / Math.max(1, totalSamples)).toFixed(1)}%</span>
        <span className="label">samples changed</span>
      </div>
      <div>
        <span className="value">
          {result.payloadBytes} / {result.capacity}
        </span>
        <span className="label">bytes used</span>
      </div>
    </div>
  );
}
