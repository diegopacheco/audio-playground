import { NOTE_NAMES } from "../audio/notes";
import type { NoteChoice, Source, Waveform } from "../types";

interface Props {
  source: Source;
  note: NoteChoice;
  detune: number;
  waveform: Waveform;
  toneFrequency: number;
  onMic: () => void;
  onTone: () => void;
  onStop: () => void;
  onNote: (note: NoteChoice) => void;
  onDetune: (cents: number) => void;
  onWaveform: (waveform: Waveform) => void;
  onOneShot: () => void;
}

const OCTAVES = [1, 2, 3, 4, 5, 6];
const WAVEFORMS: Waveform[] = ["sine", "sawtooth", "square", "triangle"];

export function SourcePanel(props: Props) {
  const { source, note, detune, waveform, toneFrequency } = props;
  return (
    <section className="panel">
      <h2>Input</h2>
      <div className="buttons">
        <button className={source === "mic" ? "active" : ""} onClick={props.onMic}>
          Microphone
        </button>
        <button className={source === "tone" ? "active" : ""} onClick={props.onTone}>
          Reference tone
        </button>
        <button onClick={props.onStop} disabled={source === "off"}>
          Stop
        </button>
      </div>
      <div className="tone-row">
        <label>
          Note
          <select
            value={note.name}
            onChange={(e) => props.onNote({ name: e.target.value, octave: note.octave })}
          >
            {NOTE_NAMES.map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </label>
        <label>
          Octave
          <select
            value={note.octave}
            onChange={(e) => props.onNote({ name: note.name, octave: Number(e.target.value) })}
          >
            {OCTAVES.map((octave) => (
              <option key={octave} value={octave}>
                {octave}
              </option>
            ))}
          </select>
        </label>
        <label>
          Wave
          <select value={waveform} onChange={(e) => props.onWaveform(e.target.value as Waveform)}>
            {WAVEFORMS.map((wave) => (
              <option key={wave} value={wave}>
                {wave}
              </option>
            ))}
          </select>
        </label>
      </div>
      <label>
        Detune {detune > 0 ? "+" : ""}
        {detune} cents
        <input
          type="range"
          min={-50}
          max={50}
          value={detune}
          onChange={(e) => props.onDetune(Number(e.target.value))}
        />
      </label>
      <p className="hint">Tone frequency {toneFrequency.toFixed(2)} Hz</p>
      <button onClick={props.onOneShot} disabled={source === "off"}>
        One-shot detect (POST)
      </button>
    </section>
  );
}
