import type { EngineStatus } from "../types";

interface Props {
  status: EngineStatus | null;
}

function Stat({ label, value, alert }: { label: string; value: string; alert?: boolean }) {
  return (
    <div className={`stat ${alert ? "alert" : ""}`}>
      <span className="stat-value">{value}</span>
      <span className="stat-label">{label}</span>
    </div>
  );
}

export function XrunPanel({ status }: Props) {
  const c = status?.counters;
  const s = status?.stream;
  const budget = status?.buffer_ms ? status.buffer_ms * 1000 : 0;
  return (
    <div className="card">
      <div className="card-head">
        <h2>Xruns and callbacks</h2>
        <span className="muted">callback budget {budget ? `${budget.toFixed(0)} us` : "-"}</span>
      </div>
      <div className="stats">
        <Stat label="underruns" value={String(c?.underruns ?? 0)} alert={(c?.underruns ?? 0) > 0} />
        <Stat label="overruns" value={String(c?.overruns ?? 0)} alert={(c?.overruns ?? 0) > 0} />
        <Stat label="sample rate" value={s ? `${s.sample_rate} Hz` : "-"} />
        <Stat label="buffer" value={s ? `${s.buffer_frames} fr / ${status?.buffer_ms?.toFixed(2)} ms` : "-"} />
      </div>
      <table className="timing">
        <thead>
          <tr>
            <th>callback</th>
            <th>count</th>
            <th>last us</th>
            <th>avg us</th>
            <th>max us</th>
          </tr>
        </thead>
        <tbody>
          {(["input_callbacks", "output_callbacks"] as const).map((k) => (
            <tr key={k}>
              <td>{k === "input_callbacks" ? "input" : "output"}</td>
              <td>{c?.[k].count ?? 0}</td>
              <td>{c?.[k].last_us.toFixed(1) ?? "-"}</td>
              <td>{c?.[k].avg_us.toFixed(1) ?? "-"}</td>
              <td>{c?.[k].max_us.toFixed(1) ?? "-"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
