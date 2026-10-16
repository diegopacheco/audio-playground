import { describe, expect, test } from "bun:test";
import { canvasToPosition, positionToCanvas, toPanner, wrapAzimuth } from "./geometry";

const center = { x: 200, y: 200 };

describe("canvas to listener geometry", () => {
  test("dragging straight up from the head means the source is in front", () => {
    const p = canvasToPosition({ x: 200, y: 80 }, center, 60, 3, 0);
    expect(p.azimuth).toBeCloseTo(0, 6);
    expect(p.distance).toBeCloseTo(2, 6);
  });

  test("dragging to the right of the head gives +90 so it matches the Rust azimuth convention", () => {
    expect(canvasToPosition({ x: 290, y: 200 }, center, 60, 3, 0).azimuth).toBeCloseTo(90, 6);
    expect(canvasToPosition({ x: 110, y: 200 }, center, 60, 3, 0).azimuth).toBeCloseTo(-90, 6);
  });

  test("a drag onto the head is pushed out to the minimum distance instead of dividing by zero", () => {
    expect(canvasToPosition({ x: 200, y: 200 }, center, 60, 3, 0).distance).toBeCloseTo(0.3, 6);
  });

  test("canvas and position conversions round trip", () => {
    const p = { azimuth: 135, elevation: 0, distance: 2.5 };
    const back = canvasToPosition(positionToCanvas(p, center, 60), center, 60, 3, 0);
    expect(back.azimuth).toBeCloseTo(135, 6);
    expect(back.distance).toBeCloseTo(2.5, 6);
  });
});

describe("Web Audio panner coordinates", () => {
  test("front maps to negative z because the listener faces -z", () => {
    const c = toPanner({ azimuth: 0, elevation: 0, distance: 1 });
    expect(c.z).toBeCloseTo(-1, 6);
    expect(c.x).toBeCloseTo(0, 6);
  });

  test("right maps to positive x and up maps to positive y", () => {
    expect(toPanner({ azimuth: 90, elevation: 0, distance: 1 }).x).toBeCloseTo(1, 6);
    expect(toPanner({ azimuth: 0, elevation: 90, distance: 1 }).y).toBeCloseTo(1, 6);
  });

  test("azimuth wraps into -180..180", () => {
    expect(wrapAzimuth(270)).toBe(-90);
    expect(wrapAzimuth(-180)).toBe(180);
    expect(wrapAzimuth(45)).toBe(45);
  });
});
