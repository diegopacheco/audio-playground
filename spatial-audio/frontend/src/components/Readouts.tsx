import type { CueReport, Position } from "../types";

interface Props {
  position: Position;
  cues: CueReport | null;
}

function Stat({ label, value, hint }: { label: string; value: string; hint: string }) {
  return (
    <div className="stat">
      <span className="stat-label">{label}</span>
      <span className="stat-value">{value}</span>
      <span className="stat-hint">{hint}</span>
    </div>
  );
}

export function Readouts({ position, cues }: Props) {
  const itd = cues ? cues.itd_ms : 0;
  const ild = cues ? cues.ild_db : 0;
  const leading = Math.abs(itd) < 0.005 ? "both ears together" : itd > 0 ? "right ear leads" : "left ear leads";
  const louder = Math.abs(ild) < 0.05 ? "equal level" : ild > 0 ? "right louder" : "left louder";
  return (
    <div className="readouts">
      <Stat label="Azimuth" value={`${position.azimuth.toFixed(1)}°`} hint="0 front, +90 right" />
      <Stat label="Elevation" value={`${position.elevation.toFixed(1)}°`} hint="+ above ear level" />
      <Stat label="Distance" value={`${position.distance.toFixed(2)} m`} hint="inverse distance gain" />
      <Stat label="ITD" value={`${itd.toFixed(3)} ms`} hint={leading} />
      <Stat
        label="ILD"
        value={`${ild.toFixed(1)} dB`}
        hint={`${louder} at ${cues ? (cues.ild_probe_hz / 1000).toFixed(0) : "4"} kHz`}
      />
    </div>
  );
}
