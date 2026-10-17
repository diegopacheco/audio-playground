import { describe, expect, test } from "bun:test";
import { envelopeOutline } from "./envelope";

describe("envelopeOutline", () => {
  test("silence collapses onto the center line", () => {
    const pts = envelopeOutline([0, 0, 0], 100, 50);
    expect(pts.every((p) => p.y === 25)).toBe(true);
  });

  test("full scale touches top and bottom edges of the canvas", () => {
    const pts = envelopeOutline([1, 1], 100, 50);
    expect(Math.min(...pts.map((p) => p.y))).toBe(0);
    expect(Math.max(...pts.map((p) => p.y))).toBe(50);
  });

  test("spans the full width so the whole file is visible", () => {
    const pts = envelopeOutline([0.2, 0.5, 0.9, 0.1], 300, 40);
    expect(pts[0].x).toBe(0);
    expect(pts[3].x).toBe(300);
  });

  test("is mirrored so it draws a closed shape around the center", () => {
    const pts = envelopeOutline([0.5, 0.8], 10, 100);
    expect(pts.length).toBe(4);
    expect(pts[0].y + pts[3].y).toBe(100);
    expect(pts[1].y + pts[2].y).toBe(100);
  });

  test("values above full scale are clamped instead of leaving the canvas", () => {
    const pts = envelopeOutline([3], 10, 20);
    expect(pts[0].y).toBe(0);
  });

  test("empty envelope draws nothing", () => {
    expect(envelopeOutline([], 10, 10)).toEqual([]);
  });
});
