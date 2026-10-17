import { describe, expect, test } from "bun:test";
import type { EffectSpec } from "../types";
import { createItem, moveItem, removeItem, setParam, toConfig } from "./chain";

const reverb: EffectSpec = {
  type: "reverb",
  label: "Reverb",
  description: "",
  params: [
    { name: "room_size", label: "Room", min: 0, max: 1, default: 0.7, step: 0.01, unit: "" },
    { name: "mix", label: "Mix", min: 0, max: 1, default: 0.35, step: 0.01, unit: "" },
  ],
};

const distortion: EffectSpec = {
  type: "distortion",
  label: "Distortion",
  description: "",
  params: [{ name: "drive", label: "Drive", min: 1, max: 50, default: 6, step: 0.1, unit: "x" }],
};

describe("chain editing", () => {
  test("new cards start from the backend defaults so the first render matches the advertised ranges", () => {
    expect(createItem(reverb, "a").params).toEqual({ room_size: 0.7, mix: 0.35 });
  });

  test("moving a card changes the order sent to the backend because order changes the sound", () => {
    const items = [createItem(reverb, "a"), createItem(distortion, "b")];
    const moved = moveItem(items, "b", -1);
    expect(toConfig(moved).map((c) => c.type)).toEqual(["distortion", "reverb"]);
  });

  test("moving past either end is ignored instead of dropping or duplicating a card", () => {
    const items = [createItem(reverb, "a"), createItem(distortion, "b")];
    expect(moveItem(items, "a", -1)).toBe(items);
    expect(moveItem(items, "b", 1)).toBe(items);
  });

  test("editing a slider only touches that card", () => {
    const items = [createItem(reverb, "a"), createItem(reverb, "b")];
    const next = setParam(items, "b", "mix", 0.9);
    expect(next[0].params.mix).toBe(0.35);
    expect(next[1].params.mix).toBe(0.9);
  });

  test("removing a card keeps the rest in order and strips ui ids from the payload", () => {
    const items = [createItem(reverb, "a"), createItem(distortion, "b"), createItem(reverb, "c")];
    const config = toConfig(removeItem(items, "b"));
    expect(config).toEqual([
      { type: "reverb", params: { room_size: 0.7, mix: 0.35 } },
      { type: "reverb", params: { room_size: 0.7, mix: 0.35 } },
    ]);
  });
});
