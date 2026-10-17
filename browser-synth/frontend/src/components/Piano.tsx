import { isBlackKey, noteName } from "../audio/notes";
import { semitoneToKey } from "../keyboard";

interface Props {
  octave: number;
  active: ReadonlySet<number>;
  onNoteOn: (midi: number) => void;
  onNoteOff: (midi: number) => void;
}

const KEY_COUNT = 24;

export function Piano({ octave, active, onNoteOn, onNoteOff }: Props) {
  const base = 12 * (octave + 1);
  const keys = Array.from({ length: KEY_COUNT }, (_, i) => base + i);
  return (
    <div className="piano">
      {keys.map((midi, i) => (
        <button
          key={midi}
          className={`key ${isBlackKey(midi) ? "black" : "white"} ${active.has(midi) ? "down" : ""}`}
          onPointerDown={(e) => {
            e.currentTarget.setPointerCapture(e.pointerId);
            onNoteOn(midi);
          }}
          onPointerUp={() => onNoteOff(midi)}
          onPointerCancel={() => onNoteOff(midi)}
        >
          <span className="hint">{semitoneToKey(i).toUpperCase()}</span>
          {!isBlackKey(midi) && <span className="name">{noteName(midi)}</span>}
        </button>
      ))}
    </div>
  );
}
