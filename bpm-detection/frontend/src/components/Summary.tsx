import type { Analysis } from "../types";

interface Props {
  analysis: Analysis;
}

export function Summary({ analysis }: Props) {
  return (
    <section className="card summary">
      <div className="stat">
        <span className="stat-value">{analysis.bpm.toFixed(1)}</span>
        <span className="label">BPM</span>
      </div>
      <div className="stat">
        <span className="stat-value">{Math.round(analysis.confidence * 100)}%</span>
        <span className="label">confidence</span>
      </div>
      <div className="stat">
        <span className="stat-value">{analysis.beats_sec.length}</span>
        <span className="label">beats</span>
      </div>
      <div className="stat">
        <span className="stat-value">{analysis.duration_sec.toFixed(1)}s</span>
        <span className="label">duration</span>
      </div>
      <div className="stat">
        <span className="stat-value">{analysis.detector}</span>
        <span className="label">detector</span>
      </div>
    </section>
  );
}
