import { describe, expect, test } from "bun:test";
import { bucketMax, project, toPath } from "./chart";

describe("bucketMax downsampling", () => {
  test("keeps narrow onset peaks that plain decimation would drop", () => {
    const values = new Array(1000).fill(0);
    values[503] = 1;
    const points = bucketMax(values, 100);
    expect(points.length).toBe(100);
    expect(points.some((p) => p.x === 503 && p.y === 1)).toBe(true);
  });

  test("short series are passed through untouched", () => {
    expect(bucketMax([3, 1, 2], 10)).toEqual([
      { x: 0, y: 3 },
      { x: 1, y: 1 },
      { x: 2, y: 2 },
    ]);
  });
});

describe("projection", () => {
  const scale = { width: 200, height: 100, xMax: 10, yMin: 0, yMax: 1 };

  test("higher values are drawn nearer the top because SVG y grows downward", () => {
    expect(project({ x: 0, y: 1 }, scale)).toEqual({ x: 0, y: 0 });
    expect(project({ x: 10, y: 0 }, scale)).toEqual({ x: 200, y: 100 });
  });

  test("path starts with a move and continues with lines", () => {
    expect(toPath([{ x: 0, y: 0 }, { x: 5, y: 0.5 }], scale)).toBe("M0.0 100.0 L100.0 50.0");
  });
});
