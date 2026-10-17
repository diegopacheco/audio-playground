import type { QueryResult } from "../types";
import { ConstellationCanvas } from "./ConstellationCanvas";
import { HistogramCanvas } from "./HistogramCanvas";

interface Props {
  result: QueryResult;
}

export function ResultPanel({ result }: Props) {
  const r = result.identification;
  const top = Math.max(1, ...r.candidates.map((c) => c.score));
  return (
    <section className="card">
      <h2>Result</h2>
      <p className="muted">Query: {result.label}</p>
      <div className={r.match ? "verdict hit" : "verdict miss"}>
        {r.match ? (
          <>
            <strong>{r.match.name}</strong>
            <span>
              at {r.offset_sec.toFixed(2)}s · score {r.score}
            </span>
          </>
        ) : (
          <>
            <strong>No match</strong>
            <span>
              best score {r.score}, needs {r.min_score}
            </span>
          </>
        )}
      </div>
      <p className="muted">
        {r.duration_sec.toFixed(1)}s analyzed · {r.peaks.length} peaks · {r.hash_count} hashes
      </p>
      <ConstellationCanvas peaks={r.peaks} duration={r.duration_sec} />
      <HistogramCanvas
        bins={r.histogram}
        winner={r.candidates[0]?.name ?? "no candidate"}
        offset={r.offset_sec}
      />
      <h3>Candidates</h3>
      <ol className="candidates">
        {r.candidates.map((c) => (
          <li key={c.track_id}>
            <span className="track-name">{c.name}</span>
            <span className="bar">
              <span style={{ width: `${(c.score / top) * 100}%` }} />
            </span>
            <span className="muted">
              {c.score} @ {c.offset_sec.toFixed(2)}s
            </span>
          </li>
        ))}
      </ol>
    </section>
  );
}
