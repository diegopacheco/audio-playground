import type { DetectorName } from "../types";

interface Props {
  detector: DetectorName;
  a4: number;
  onDetector: (detector: DetectorName) => void;
  onA4: (a4: number) => void;
}

export function SettingsPanel({ detector, a4, onDetector, onA4 }: Props) {
  return (
    <section className="panel">
      <h2>Settings</h2>
      <label>
        Detector
        <select value={detector} onChange={(e) => onDetector(e.target.value as DetectorName)}>
          <option value="yin">YIN</option>
          <option value="autocorrelation">Autocorrelation</option>
        </select>
      </label>
      <label>
        A4 reference (Hz)
        <input
          type="number"
          min={400}
          max={480}
          step={0.5}
          value={a4}
          onChange={(e) => {
            const value = Number(e.target.value);
            if (value > 0) {
              onA4(value);
            }
          }}
        />
      </label>
    </section>
  );
}
