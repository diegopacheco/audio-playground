import { BUFFER_SIZES } from "../latency";
import type { Mode, ProbeName } from "../types";

interface Props {
  mode: Mode;
  bufferFrames: number;
  probe: ProbeName;
  running: boolean;
  busy: boolean;
  onMode: (m: Mode) => void;
  onBuffer: (n: number) => void;
  onProbe: (p: ProbeName) => void;
  onStart: () => void;
  onStop: () => void;
  onMeasure: () => void;
}

export function EngineControls(p: Props) {
  return (
    <div className="controls">
      <label className="field">
        <span>Mode</span>
        <select value={p.mode} disabled={p.running} onChange={(e) => p.onMode(e.target.value as Mode)}>
          <option value="passthrough">Passthrough (monitor mic)</option>
          <option value="measure">Measure (silent until probe)</option>
        </select>
      </label>
      <label className="field">
        <span>Buffer frames</span>
        <select
          value={p.bufferFrames}
          disabled={p.running}
          onChange={(e) => p.onBuffer(Number(e.target.value))}
        >
          {BUFFER_SIZES.map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </select>
      </label>
      <label className="field">
        <span>Probe</span>
        <select value={p.probe} disabled={p.running} onChange={(e) => p.onProbe(e.target.value as ProbeName)}>
          <option value="correlation">Chirp + cross-correlation</option>
          <option value="threshold">Click + threshold</option>
        </select>
      </label>
      <div className="buttons">
        {p.running ? (
          <button className="secondary" disabled={p.busy} onClick={p.onStop}>
            Stop
          </button>
        ) : (
          <button disabled={p.busy} onClick={p.onStart}>
            Start
          </button>
        )}
        <button disabled={!p.running || p.busy} onClick={p.onMeasure}>
          Measure latency
        </button>
      </div>
    </div>
  );
}
