import type { Preset, Waveform } from "../types";
import { WAVEFORMS } from "../types";
import { FilterCurve } from "./FilterCurve";

interface Props {
  preset: Preset;
  onChange: (preset: Preset) => void;
}

type NumericKey = "attack" | "decay" | "sustain" | "release" | "cutoff" | "resonance" | "gain";

interface Slider {
  key: NumericKey;
  label: string;
  min: number;
  max: number;
  step: number;
  unit: string;
}

const ENVELOPE: Slider[] = [
  { key: "attack", label: "Attack", min: 0, max: 2, step: 0.001, unit: "s" },
  { key: "decay", label: "Decay", min: 0, max: 2, step: 0.001, unit: "s" },
  { key: "sustain", label: "Sustain", min: 0, max: 1, step: 0.01, unit: "" },
  { key: "release", label: "Release", min: 0, max: 4, step: 0.001, unit: "s" },
];

const TONE: Slider[] = [
  { key: "cutoff", label: "Cutoff", min: 20, max: 20000, step: 1, unit: "Hz" },
  { key: "resonance", label: "Resonance", min: -10, max: 30, step: 0.1, unit: "dB" },
  { key: "gain", label: "Gain", min: 0, max: 1, step: 0.01, unit: "" },
];

export function Controls({ preset, onChange }: Props) {
  const slider = (s: Slider) => (
    <label key={s.key} className="slider">
      <span>
        {s.label}
        <em>
          {preset[s.key]}
          {s.unit}
        </em>
      </span>
      <input
        type="range"
        min={s.min}
        max={s.max}
        step={s.step}
        value={preset[s.key]}
        onChange={(e) => onChange({ ...preset, [s.key]: Number(e.target.value) })}
      />
    </label>
  );

  return (
    <section className="panel controls">
      <div className="group">
        <h2>Oscillator</h2>
        <div className="waveforms">
          {WAVEFORMS.map((w: Waveform) => (
            <button
              key={w}
              className={w === preset.waveform ? "active" : ""}
              onClick={() => onChange({ ...preset, waveform: w })}
            >
              {w}
            </button>
          ))}
        </div>
      </div>
      <div className="group">
        <h2>Envelope</h2>
        {ENVELOPE.map(slider)}
      </div>
      <div className="group">
        <h2>Filter</h2>
        {TONE.map(slider)}
        <FilterCurve cutoff={preset.cutoff} resonance={preset.resonance} />
      </div>
    </section>
  );
}
