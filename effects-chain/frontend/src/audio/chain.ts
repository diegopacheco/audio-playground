import type { ChainItem, EffectConfig, EffectSpec } from "../types";

export function createItem(spec: EffectSpec, id: string): ChainItem {
  const params: Record<string, number> = {};
  for (const p of spec.params) params[p.name] = p.default;
  return { id, type: spec.type, params };
}

export function removeItem(items: ChainItem[], id: string): ChainItem[] {
  return items.filter((item) => item.id !== id);
}

export function moveItem(items: ChainItem[], id: string, delta: number): ChainItem[] {
  const from = items.findIndex((item) => item.id === id);
  const to = from + delta;
  if (from < 0 || to < 0 || to >= items.length) return items;
  const next = [...items];
  const [moved] = next.splice(from, 1);
  next.splice(to, 0, moved);
  return next;
}

export function setParam(items: ChainItem[], id: string, name: string, value: number): ChainItem[] {
  return items.map((item) =>
    item.id === id ? { ...item, params: { ...item.params, [name]: value } } : item,
  );
}

export function toConfig(items: ChainItem[]): EffectConfig[] {
  return items.map(({ type, params }) => ({ type, params }));
}
