import type { GenerateRequest, Generator, Options } from "../types";

interface Props {
  options: Options;
  request: GenerateRequest;
  onChange: (request: GenerateRequest) => void;
}

export function Controls({ options, request, onChange }: Props) {
  const set = <K extends keyof GenerateRequest>(field: K, value: GenerateRequest[K]) =>
    onChange({ ...request, [field]: value });

  return (
    <div className="controls">
      <label>
        Key
        <select value={request.key} onChange={(e) => set("key", e.target.value)}>
          {options.keys.map((k) => (
            <option key={k}>{k}</option>
          ))}
        </select>
      </label>
      <label>
        Scale
        <select value={request.scale} onChange={(e) => set("scale", e.target.value)}>
          {options.scales.map((s) => (
            <option key={s}>{s}</option>
          ))}
        </select>
      </label>
      <label className="wide">
        Progression
        <input list="progressions" value={request.progression} onChange={(e) => set("progression", e.target.value)} />
        <datalist id="progressions">
          {options.progressions.map((p) => (
            <option key={p} value={p} />
          ))}
        </datalist>
      </label>
      <label>
        Generator
        <select value={request.generator} onChange={(e) => set("generator", e.target.value as Generator)}>
          {options.generators.map((g) => (
            <option key={g}>{g}</option>
          ))}
        </select>
      </label>
      <label>
        Order
        <select
          value={request.order}
          disabled={request.generator !== "markov"}
          onChange={(e) => set("order", Number(e.target.value))}
        >
          {options.orders.map((o) => (
            <option key={o} value={o}>
              {o}
            </option>
          ))}
        </select>
      </label>
      <label>
        Seed
        <input type="number" min={0} value={request.seed} onChange={(e) => set("seed", Number(e.target.value))} />
      </label>
      <label>
        BPM
        <input
          type="number"
          min={options.min_bpm}
          max={options.max_bpm}
          value={request.bpm}
          onChange={(e) => set("bpm", Number(e.target.value))}
        />
      </label>
      <label>
        Bars
        <input
          type="number"
          min={1}
          max={options.max_bars}
          value={request.bars}
          onChange={(e) => set("bars", Number(e.target.value))}
        />
      </label>
    </div>
  );
}
