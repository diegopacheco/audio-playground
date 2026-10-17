import type { Settings, SourceKind, WindowName } from "../types";

const WINDOWS: WindowName[] = ["hann", "hamming", "rect"];
const FFT_SIZES = [512, 1024, 2048, 4096, 8192];

interface Props {
  settings: Settings;
  running: boolean;
  onChange: (next: Settings) => void;
  onStart: () => void;
  onStop: () => void;
  onSnapshot: () => void;
}

export function Controls({ settings, running, onChange, onStart, onStop, onSnapshot }: Props) {
  return (
    <section className="controls">
      <label>
        Source
        <select
          value={settings.source}
          onChange={(e) => onChange({ ...settings, source: e.target.value as SourceKind })}
        >
          <option value="tone">Test tone</option>
          <option value="mic">Microphone</option>
        </select>
      </label>
      <label>
        Window
        <select
          value={settings.window}
          onChange={(e) => onChange({ ...settings, window: e.target.value as WindowName })}
        >
          {WINDOWS.map((w) => (
            <option key={w} value={w}>
              {w}
            </option>
          ))}
        </select>
      </label>
      <label>
        FFT size
        <select
          value={settings.fftSize}
          onChange={(e) => onChange({ ...settings, fftSize: Number(e.target.value) })}
        >
          {FFT_SIZES.map((n) => (
            <option key={n} value={n}>
              {n}
            </option>
          ))}
        </select>
      </label>
      <label className={settings.source === "tone" ? "" : "disabled"}>
        Tone {Math.round(settings.toneHz)} Hz
        <input
          type="range"
          min={1.3}
          max={4.3}
          step={0.001}
          disabled={settings.source !== "tone"}
          value={Math.log10(settings.toneHz)}
          onChange={(e) => onChange({ ...settings, toneHz: 10 ** Number(e.target.value) })}
        />
      </label>
      <div className="buttons">
        {running ? (
          <button onClick={onStop}>Stop</button>
        ) : (
          <button className="primary" onClick={onStart}>
            Start
          </button>
        )}
        <button onClick={onSnapshot} disabled={!running}>
          Snapshot
        </button>
      </div>
    </section>
  );
}
