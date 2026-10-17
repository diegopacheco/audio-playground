import type { Method, SourceKind, Trajectory, TrajectoryInfo, TrajectoryType } from "../types";

interface Props {
  trajectories: TrajectoryInfo[];
  source: SourceKind;
  method: Method;
  duration: number;
  busy: boolean;
  playing: TrajectoryType | null;
  onSource: (source: SourceKind) => void;
  onMethod: (method: Method) => void;
  onDuration: (duration: number) => void;
  onRender: (trajectory: Trajectory) => void;
  onStop: () => void;
  staticTrajectory: Trajectory;
}

const SOURCES: SourceKind[] = ["noise", "tone", "clicks", "voice-like"];

function summary(t: Trajectory): string {
  switch (t.type) {
    case "circle":
      return `period ${t.period} s, radius ${t.distance} m`;
    case "figure8":
      return `period ${t.period} s, size ${t.size} m`;
    case "static":
      return `az ${t.azimuth.toFixed(0)}°, el ${t.elevation.toFixed(0)}°, ${t.distance.toFixed(2)} m (from the head view)`;
    case "flyby":
      return `${t.speed} m/s, closest ${t.closest} m`;
  }
}

export function RenderPanel(props: Props) {
  const { trajectories, source, method, duration, busy, playing } = props;
  return (
    <section className="panel">
      <h2>Render in Rust</h2>
      <div className="controls">
        <label>
          Source
          <select value={source} onChange={(e) => props.onSource(e.target.value as SourceKind)}>
            {SOURCES.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </label>
        <label>
          Method
          <select value={method} onChange={(e) => props.onMethod(e.target.value as Method)}>
            <option value="binaural">Simple binaural (ITD + head shadow)</option>
            <option value="hrtf">Parametric HRTF (+ pinna notch, back tilt)</option>
          </select>
        </label>
        <label>
          Duration {duration.toFixed(0)} s
          <input
            type="range"
            min={2}
            max={16}
            step={1}
            value={duration}
            onChange={(e) => props.onDuration(Number(e.target.value))}
          />
        </label>
      </div>
      <div className="cards">
        {trajectories.map((info) => {
          const trajectory = info.defaults.type === "static" ? props.staticTrajectory : info.defaults;
          const active = playing === trajectory.type;
          return (
            <div key={info.label} className={active ? "card active" : "card"}>
              <h3>{info.label}</h3>
              <p className="muted small">{info.description}</p>
              <p className="small mono">{summary(trajectory)}</p>
              {active ? (
                <button className="secondary" onClick={props.onStop}>
                  Stop
                </button>
              ) : (
                <button disabled={busy} onClick={() => props.onRender(trajectory)}>
                  Render in Rust and play
                </button>
              )}
            </div>
          );
        })}
      </div>
    </section>
  );
}
