import { formatDb, formatHz } from "../audio/format";
import type { SpectrumFrame } from "../types";

interface Props {
  frame: SpectrumFrame | null;
  snapshot: SpectrumFrame | null;
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="stat">
      <span className="label">{label}</span>
      <span className="value">{value}</span>
    </div>
  );
}

export function PeakReadout({ frame, snapshot }: Props) {
  return (
    <section className="readout">
      <Stat label="Peak frequency" value={frame ? formatHz(frame.peak_hz) : "-"} />
      <Stat label="Peak level" value={frame ? formatDb(frame.peak_db) : "-"} />
      <Stat
        label="Bin width"
        value={frame ? formatHz(frame.sample_rate / frame.fft_size) : "-"}
      />
      <Stat
        label="Snapshot peak"
        value={snapshot ? `${formatHz(snapshot.peak_hz)} / ${formatDb(snapshot.peak_db)}` : "-"}
      />
    </section>
  );
}
