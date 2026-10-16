import type { SummaryMode, SummaryResult } from "../types";

interface Props {
  enabled: boolean;
  busy: boolean;
  result: SummaryResult | null;
  error: string | null;
  onRun: (mode: SummaryMode) => void;
}

export function SummaryPanel({ enabled, busy, result, error, onRun }: Props) {
  return (
    <section className="card">
      <h2>Claude</h2>
      <div className="row">
        <button className="primary" disabled={!enabled || busy} onClick={() => onRun("summary")}>
          Summary
        </button>
        <button className="primary" disabled={!enabled || busy} onClick={() => onRun("action_items")}>
          Action items
        </button>
        {busy && <span className="muted">asking Claude...</span>}
      </div>
      {error && <p className="error">{error}</p>}
      {result && (
        <div className="result">
          {result.summary && <p>{result.summary}</p>}
          {result.action_items.length > 0 && (
            <ol>
              {result.action_items.map((item, i) => (
                <li key={i}>{item}</li>
              ))}
            </ol>
          )}
        </div>
      )}
    </section>
  );
}
