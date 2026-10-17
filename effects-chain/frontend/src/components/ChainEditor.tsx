import type { ChainItem, EffectSpec } from "../types";
import { EffectCard } from "./EffectCard";

interface Props {
  specs: EffectSpec[];
  items: ChainItem[];
  onAdd: (spec: EffectSpec) => void;
  onParam: (id: string, name: string, value: number) => void;
  onMove: (id: string, delta: number) => void;
  onRemove: (id: string) => void;
}

export function ChainEditor({ specs, items, onAdd, onParam, onMove, onRemove }: Props) {
  return (
    <section className="panel">
      <h2>Chain</h2>
      <div className="palette">
        {specs.map((spec) => (
          <button key={spec.type} onClick={() => onAdd(spec)}>
            + {spec.label}
          </button>
        ))}
      </div>
      {items.length === 0 && <p className="muted">No effects yet. The signal passes through untouched.</p>}
      <div className="cards">
        {items.map((item, index) => {
          const spec = specs.find((s) => s.type === item.type);
          return spec ? (
            <EffectCard
              key={item.id}
              item={item}
              spec={spec}
              index={index}
              total={items.length}
              onParam={onParam}
              onMove={onMove}
              onRemove={onRemove}
            />
          ) : null;
        })}
      </div>
    </section>
  );
}
