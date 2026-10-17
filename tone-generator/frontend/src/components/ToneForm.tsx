import type { ToneRequest, Waveform } from "../types";

interface Props {
  value: ToneRequest;
  waveforms: Waveform[];
  sampleRates: number[];
  busy: boolean;
  onChange: (next: ToneRequest) => void;
  onSubmit: () => void;
}

export function ToneForm({ value, waveforms, sampleRates, busy, onChange, onSubmit }: Props) {
  const set = <K extends keyof ToneRequest>(key: K, v: ToneRequest[K]) => onChange({ ...value, [key]: v });
  return (
    <form
      className="card form"
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit();
      }}
    >
      <div className="waveforms">
        {waveforms.map((w) => (
          <button
            type="button"
            key={w}
            className={w === value.waveform ? "chip active" : "chip"}
            onClick={() => set("waveform", w)}
          >
            {w}
          </button>
        ))}
      </div>
      <label>
        Frequency <b>{value.frequency} Hz</b>
        <input
          type="range"
          min={20}
          max={2000}
          value={value.frequency}
          onChange={(e) => set("frequency", Number(e.target.value))}
        />
      </label>
      <label>
        Duration <b>{value.duration_ms} ms</b>
        <input
          type="range"
          min={100}
          max={5000}
          step={100}
          value={value.duration_ms}
          onChange={(e) => set("duration_ms", Number(e.target.value))}
        />
      </label>
      <label>
        Amplitude <b>{value.amplitude.toFixed(2)}</b>
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={value.amplitude}
          onChange={(e) => set("amplitude", Number(e.target.value))}
        />
      </label>
      <label>
        Sample rate
        <select value={value.sample_rate} onChange={(e) => set("sample_rate", Number(e.target.value))}>
          {sampleRates.map((r) => (
            <option key={r} value={r}>
              {r} Hz
            </option>
          ))}
        </select>
      </label>
      <button type="submit" className="primary" disabled={busy}>
        {busy ? "Generating..." : "Generate tone"}
      </button>
    </form>
  );
}
