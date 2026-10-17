import type { ChainItem, EffectSpec } from "../types";

interface Props {
  item: ChainItem;
  spec: EffectSpec;
  index: number;
  total: number;
  onParam: (id: string, name: string, value: number) => void;
  onMove: (id: string, delta: number) => void;
  onRemove: (id: string) => void;
}

export function EffectCard({ item, spec, index, total, onParam, onMove, onRemove }: Props) {
  return (
    <div className="card">
      <div className="card-head">
        <span className="badge">{index + 1}</span>
        <strong>{spec.label}</strong>
        <div className="card-actions">
          <button aria-label="move up" disabled={index === 0} onClick={() => onMove(item.id, -1)}>
            &uarr;
          </button>
          <button aria-label="move down" disabled={index === total - 1} onClick={() => onMove(item.id, 1)}>
            &darr;
          </button>
          <button aria-label="remove" className="danger" onClick={() => onRemove(item.id)}>
            &times;
          </button>
        </div>
      </div>
      <p className="muted small">{spec.description}</p>
      {spec.params.map((p) => (
        <label key={p.name} className="slider">
          <span>{p.label}</span>
          <input
            type="range"
            min={p.min}
            max={p.max}
            step={p.step}
            value={item.params[p.name]}
            onChange={(e) => onParam(item.id, p.name, Number(e.target.value))}
          />
          <span className="value">
            {item.params[p.name]}
            {p.unit && ` ${p.unit}`}
          </span>
        </label>
      ))}
    </div>
  );
}
