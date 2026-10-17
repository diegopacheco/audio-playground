import { isInTune } from "../audio/notes";
import type { DetectResponse } from "../types";

interface Props {
  reading: DetectResponse | null;
}

export function NoteDisplay({ reading }: Props) {
  const hasPitch = reading !== null && reading.note !== null;
  const cents = reading?.cents ?? null;
  return (
    <div className="note-display">
      <div className={isInTune(cents) ? "note note-tuned" : "note"}>
        {hasPitch ? reading.note : "--"}
        <span className="octave">{hasPitch ? reading.octave : ""}</span>
      </div>
      <div className="readout">
        <span>{reading?.frequency != null ? `${reading.frequency.toFixed(2)} Hz` : "no pitch"}</span>
        <span>{cents !== null ? `${cents > 0 ? "+" : ""}${cents.toFixed(1)} cents` : ""}</span>
        <span>clarity {reading ? reading.clarity.toFixed(2) : "0.00"}</span>
      </div>
    </div>
  );
}
