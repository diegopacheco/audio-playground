import { jitterStats } from "../audio/stats";
import type { BeatSample } from "../types";

interface Props {
  title: string;
  description: string;
  color: string;
  running: boolean;
  beatsPerBar: number;
  samples: BeatSample[];
  onToggle: () => void;
}

export function MetronomePanel({ title, description, color, running, beatsPerBar, samples, onToggle }: Props) {
  const stats = jitterStats(samples.map((s) => s.errorMs));
  const last = samples[samples.length - 1];
  const current = running && last ? last.index % beatsPerBar : -1;

  return (
    <section className="card panel" style={{ borderTopColor: color }}>
      <header>
        <h2>{title}</h2>
        <button type="button" className={running ? "stop" : "start"} onClick={onToggle}>
          {running ? "Stop" : "Start"}
        </button>
      </header>
      <p className="muted">{description}</p>
      <div className="beats">
        {Array.from({ length: beatsPerBar }, (_, i) => (
          <span
            key={i}
            className={`dot ${i === 0 ? "accent" : ""}`}
            style={i === current ? { background: color, borderColor: color } : undefined}
          />
        ))}
      </div>
      <dl className="stats">
        <div><dt>beats</dt><dd>{stats.count}</dd></div>
        <div><dt>mean error</dt><dd>{stats.meanMs.toFixed(2)} ms</dd></div>
        <div><dt>jitter (std dev)</dt><dd>{stats.stdDevMs.toFixed(2)} ms</dd></div>
        <div><dt>worst</dt><dd>{stats.maxAbsMs.toFixed(2)} ms</dd></div>
      </dl>
    </section>
  );
}
