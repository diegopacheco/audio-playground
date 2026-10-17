import { useState } from "react";
import { fetchDrift } from "../api";
import type { DriftReport } from "../types";
import { JitterChart } from "./JitterChart";

interface Props {
  bpm: number;
  naiveColor: string;
  absoluteColor: string;
}

export function DriftPanel({ bpm, naiveColor, absoluteColor }: Props) {
  const [beats, setBeats] = useState(32);
  const [report, setReport] = useState<DriftReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const seconds = ((beats - 1) * 60) / bpm;

  const run = async () => {
    setBusy(true);
    setError(null);
    try {
      setReport(await fetchDrift(bpm, beats));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="card">
      <h2>Rust scheduler drift</h2>
      <p className="muted">
        The backend really runs tokio sleeps: a naive loop that sleeps one interval after each beat, and an absolute-deadline loop that sleeps until start + n x interval.
      </p>
      <div className="row">
        <label>
          Beats
          <input type="number" min={2} max={64} value={beats} onChange={(e) => setBeats(Number(e.target.value))} />
        </label>
        <button type="button" onClick={run} disabled={busy}>
          {busy ? `Running ~${seconds.toFixed(1)} s...` : `Measure at ${bpm} bpm`}
        </button>
      </div>
      {error && <p className="error">{error}</p>}
      {report && (
        <>
          <dl className="stats">
            <div><dt>interval</dt><dd>{report.interval_ms.toFixed(2)} ms</dd></div>
            <div><dt>{report.naive} worst</dt><dd>{report.naive_max_drift_ms.toFixed(3)} ms</dd></div>
            <div><dt>{report.absolute} worst</dt><dd>{report.absolute_max_drift_ms.toFixed(3)} ms</dd></div>
          </dl>
          <JitterChart
            title="Drift from ideal beat time"
            unit="ms"
            series={[
              { name: report.naive, color: naiveColor, values: report.beats.map((b) => b.naive_drift_ms) },
              { name: report.absolute, color: absoluteColor, values: report.beats.map((b) => b.absolute_drift_ms) },
            ]}
          />
          <details>
            <summary>Per-beat table</summary>
            <table>
              <thead>
                <tr><th>beat</th><th>ideal ms</th><th>naive ms</th><th>naive drift</th><th>absolute ms</th><th>absolute drift</th></tr>
              </thead>
              <tbody>
                {report.beats.map((b) => (
                  <tr key={b.index}>
                    <td>{b.index}</td>
                    <td>{b.ideal_ms.toFixed(2)}</td>
                    <td>{b.naive_ms.toFixed(2)}</td>
                    <td>{b.naive_drift_ms.toFixed(3)}</td>
                    <td>{b.absolute_ms.toFixed(2)}</td>
                    <td>{b.absolute_drift_ms.toFixed(3)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </details>
        </>
      )}
    </section>
  );
}
