export interface ParamSpec {
  name: string;
  label: string;
  min: number;
  max: number;
  default: number;
  step: number;
  unit: string;
}

export interface EffectSpec {
  type: string;
  label: string;
  description: string;
  params: ParamSpec[];
}

export interface EffectConfig {
  type: string;
  params: Record<string, number>;
}

export interface ChainItem extends EffectConfig {
  id: string;
}

export interface Clip {
  name: string;
  bytes: ArrayBuffer;
}

export interface Peak {
  min: number;
  max: number;
}

export type Side = "A" | "B";
