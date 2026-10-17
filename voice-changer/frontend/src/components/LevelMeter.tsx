import { meterPercent } from "../audio/metrics";

interface Props {
  label: string;
  db: number;
}

export function LevelMeter({ label, db }: Props) {
  const percent = meterPercent(db);
  const tone = db > -6 ? "hot" : db > -24 ? "good" : "low";
  return (
    <div className="meter">
      <div className="meter-head">
        <span>{label}</span>
        <span className="mono">{db.toFixed(1)} dBFS</span>
      </div>
      <div className="meter-track">
        <div className={`meter-fill meter-${tone}`} style={{ width: `${percent}%` }} />
      </div>
    </div>
  );
}
